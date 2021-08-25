use crate::frames::{get_frames, Frame, Metadata};
use crate::throw;
use base64;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::api::dialog;
use tauri::{command, State};

#[derive(Debug, Clone)]
pub struct Item {
  path: PathBuf,
  metadata: Metadata,
}

#[derive(Debug, Default)]
pub struct App {
  open_item: Option<Item>,
}

#[derive(Default)]
pub struct Data(pub Arc<Mutex<App>>);

#[command]
pub fn error_popup(msg: String, win: tauri::Window) {
  println!("Error popup: {}", msg);
  thread::spawn(move || {
    dialog::message(Some(&win), "Error", msg);
  });
}

fn get_metadata(path: &PathBuf) -> Result<Metadata, String> {
  let ext = path.extension().unwrap_or_default().to_string_lossy();
  let metadata = match ext.as_ref() {
    "mp3" | "aiff" | "wav" => {
      let tag = match id3::Tag::read_from_path(&path) {
        Ok(tag) => tag,
        Err(_) => id3::Tag::new(),
      };
      // for frame in tag.frames() {
      //   println!("MP3 FRAME {:?}", frame);
      // }
      Metadata::Id3(tag)
    }
    "m4a" | "mp4" | "m4p" | "m4b" | "m4r" | "m4v" => {
      let tag = match mp4ameta::Tag::read_from_path(&path) {
        Ok(tag) => tag,
        Err(_) => throw!("No tags found"),
      };
      // for (ident, data) in tag.data() {
      //   println!("M4A FRAME {:?}, {:?}", ident, data);
      // }
      Metadata::Mp4(tag)
    }
    _ => throw!("Unsupported file type"),
  };
  Ok(metadata)
}

#[derive(Serialize)]
pub struct Image {
  data: String,
  mime_type: String,
}

#[derive(Serialize)]
pub struct Info {
  path: PathBuf,
  artwork: Option<Image>,
  frames: Vec<Frame>,
}

#[command]
pub async fn open(path: PathBuf, app: State<'_, Data>) -> Result<Option<Info>, String> {
  let mut app = app.0.lock().unwrap();
  let metadata = get_metadata(&path)?;
  app.open_item = Some(Item {
    path: path.clone(),
    metadata: metadata.clone(),
  });
  let info = Info {
    path,
    artwork: match metadata {
      Metadata::Id3(ref tag) => {
        let mut img = match tag.pictures().next() {
          Some(pic) => Some(Image {
            data: base64::encode(&pic.data),
            mime_type: pic.mime_type.clone(),
          }),
          None => None,
        };
        for pic in tag.pictures() {
          if pic.picture_type == id3::frame::PictureType::CoverFront {
            img = Some(Image {
              data: base64::encode(&pic.data),
              mime_type: pic.mime_type.clone(),
            });
          }
        }
        img
      }
      Metadata::Mp4(ref tag) => match tag.artwork() {
        Some(artwork) => Some(Image {
          data: base64::encode(&artwork.data),
          mime_type: match artwork.fmt {
            mp4ameta::ImgFmt::Bmp => "BMP".to_string(),
            mp4ameta::ImgFmt::Jpeg => "JPEG".to_string(),
            mp4ameta::ImgFmt::Png => "PNG".to_string(),
          },
        }),
        None => None,
      },
    },
    frames: get_frames(&metadata),
  };
  Ok(Some(info))
}
