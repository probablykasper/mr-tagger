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
  item: Option<Item>,
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
pub struct Info {
  path: PathBuf,
  frames: Vec<Frame>,
}

#[command]
pub async fn open(path: PathBuf, app: State<'_, Data>) -> Result<Option<Info>, String> {
  let mut app = app.0.lock().unwrap();
  let metadata = get_metadata(&path)?;
  app.item = Some(Item {
    path: path.clone(),
    metadata: metadata.clone(),
  });
  let info = Info {
    path,
    frames: get_frames(&metadata),
  };
  Ok(Some(info))
}

#[derive(Serialize)]
pub struct Image {
  index: usize,
  total_images: usize,
  data: String,
  mime_type: String,
}

#[command]
pub fn get_image(index: Option<usize>, app: State<'_, Data>) -> Result<Option<Image>, String> {
  let app = app.0.lock().unwrap();
  let item = app.item.as_ref().unwrap();
  let index = match index {
    Some(index) => index,
    None => match item.metadata {
      Metadata::Id3(ref tag) => {
        let mut index = match tag.pictures().next() {
          Some(_pic) => 0,
          None => return Ok(None),
        };
        for (i, current_pic) in tag.pictures().enumerate() {
          if current_pic.picture_type == id3::frame::PictureType::CoverFront {
            index = i;
            break;
          }
        }
        index
      }
      Metadata::Mp4(ref tag) => match tag.artwork() {
        Some(_artwork) => 0,
        None => return Ok(None),
      },
    },
  };
  let image = match item.metadata {
    Metadata::Id3(ref tag) => match tag.pictures().nth(index) {
      Some(pic) => Some(Image {
        index,
        total_images: tag.pictures().count(),
        data: base64::encode(&pic.data),
        mime_type: pic.mime_type.clone(),
      }),
      None => None,
    },
    Metadata::Mp4(ref tag) => match tag.artworks().nth(index) {
      Some(artwork) => Some(Image {
        index,
        total_images: tag.artworks().count(),
        data: base64::encode(&artwork.data),
        mime_type: match artwork.fmt {
          mp4ameta::ImgFmt::Bmp => "image/bmp".to_string(),
          mp4ameta::ImgFmt::Jpeg => "image/jpeg".to_string(),
          mp4ameta::ImgFmt::Png => "image/png".to_string(),
        },
      }),
      None => None,
    },
  };
  Ok(image)
}

#[command]
pub fn remove_image(index: usize, app: State<'_, Data>) -> Result<(), String> {
  let mut app = app.0.lock().unwrap();
  let item = match &mut app.item {
    Some(item) => item,
    None => throw!("No open item"),
  };
  match item.metadata {
    Metadata::Id3(ref mut tag) => {
      let mut pic_frames: Vec<_> = tag
        .frames()
        .filter(|frame| frame.content().picture().is_some())
        .map(|frame| frame.clone())
        .collect();
      pic_frames.remove(index);
      tag.remove_all_pictures();
      for pic_frame in pic_frames {
        tag.add_frame(pic_frame);
      }
    }
    Metadata::Mp4(ref mut tag) => {
      let mut artworks: Vec<_> = tag.take_artworks().collect();
      artworks.remove(index);
      tag.set_artworks(artworks);
    }
  }
  Ok(())
}
