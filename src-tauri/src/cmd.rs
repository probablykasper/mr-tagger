use crate::frames::{get_frames, Frame, Metadata};
use crate::throw;
use base64;
use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::api::dialog;
use tauri::{command, State};

#[derive(Debug, Clone, Serialize)]
pub struct File {
  path: PathBuf,
  dirty: bool,
  #[serde(skip_serializing)]
  metadata: Metadata,
}

#[derive(Debug, Default, Serialize)]
pub struct App {
  current_index: usize,
  files: Vec<File>,
}
impl App {
  fn current_file(&mut self) -> Result<&mut File, String> {
    match self.files.get_mut(self.current_index) {
      Some(file) => Ok(file),
      None => {
        throw!("Error getting open file")
      }
    }
  }
}

#[derive(Default)]
pub struct AppState(pub Arc<Mutex<App>>);

type AppArg<'a> = State<'a, AppState>;

#[command]
pub fn error_popup(msg: String, win: tauri::Window) {
  println!("Error popup: {}", msg);
  thread::spawn(move || {
    dialog::message(Some(&win), "Error", msg);
  });
}

#[command]
pub fn get_app(app: AppArg<'_>) -> Value {
  let app = app.0.lock().unwrap();
  serde_json::to_value(&*app).unwrap()
}

fn get_metadata(path: &PathBuf) -> Result<Metadata, String> {
  let ext = path.extension().unwrap_or_default().to_string_lossy();
  let path_str = path.to_string_lossy();
  let metadata = match ext.as_ref() {
    "mp3" | "aiff" => {
      let tag = match id3::Tag::read_from_path(&path) {
        Ok(tag) => tag,
        Err(e) => match e.kind {
          id3::ErrorKind::NoTag => id3::Tag::default(),
          _ => throw!("Error reading tag for file {}: {}", path_str, e.description),
        },
      };
      Metadata::Id3(tag)
    }
    "m4a" | "mp4" | "m4p" | "m4b" | "m4r" | "m4v" => {
      let tag = match mp4ameta::Tag::read_from_path(&path) {
        Ok(tag) => tag,
        Err(e) => match e.kind {
          mp4ameta::ErrorKind::NoTag => mp4ameta::Tag::default(),
          _ => throw!("Error reading tag for file {}: {}", path_str, e.description),
        },
      };
      Metadata::Mp4(tag)
    }
    _ => throw!("Unsupported file type"),
  };
  Ok(metadata)
}

#[command]
pub async fn open_files(paths: Vec<PathBuf>, app: AppArg<'_>) -> Result<(), String> {
  let mut app = app.0.lock().unwrap();
  let initial_len = app.files.len();
  for path in paths {
    let is_duplicate = app.files.iter().any(|f| f.path == path);
    if !is_duplicate {
      let metadata = get_metadata(&path)?;
      app.files.push(File {
        path: path.clone(),
        dirty: false,
        metadata: metadata.clone(),
      });
    }
  }
  if initial_len == 0 && app.files.len() >= 1 {
    app.current_index = app.files.len() - 1;
  }
  Ok(())
}

#[command]
pub async fn close_file(index: usize, app: AppArg<'_>) -> Result<(), String> {
  let mut app = app.0.lock().unwrap();
  app.files.remove(index);
  if app.current_index >= index && index >= 1 {
    app.current_index -= 1;
  }
  Ok(())
}

#[command]
pub fn show(index: usize, app: AppArg<'_>) {
  let mut app = app.0.lock().unwrap();
  app.current_index = index;
}

#[derive(Serialize)]
pub struct Page {
  path: PathBuf,
  frames: Vec<Frame>,
}

#[command]
pub fn get_page(app: AppArg<'_>) -> Option<Page> {
  let mut app = app.0.lock().unwrap();
  let file = app.current_file().ok()?;
  Some(Page {
    path: file.path.clone(),
    frames: get_frames(&file.metadata),
  })
}

#[derive(Serialize)]
pub struct Image {
  index: usize,
  total_images: usize,
  data: String,
  mime_type: String,
}

#[command]
pub fn get_image(index: Option<usize>, app: AppArg<'_>) -> Option<Image> {
  let mut app = app.0.lock().unwrap();
  let file = app.current_file().ok()?;
  let index = match index {
    Some(index) => index,
    None => match file.metadata {
      Metadata::Id3(ref tag) => {
        let mut index = match tag.pictures().next() {
          Some(_pic) => 0,
          None => return None,
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
        None => return None,
      },
    },
  };
  match file.metadata {
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
  }
}

#[command]
pub fn remove_image(index: usize, app: AppArg<'_>) -> Result<(), String> {
  let mut app = app.0.lock().unwrap();
  let file = app.current_file().unwrap();
  file.dirty = true;
  match file.metadata {
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

#[command]
pub fn replace_image(index: usize, path: PathBuf, app: AppArg<'_>) -> Result<(), String> {
  let mut app = app.0.lock().unwrap();
  let file = app.current_file().unwrap();
  let new_bytes = match fs::read(&path) {
    Ok(b) => b,
    Err(e) => throw!("Error reading that file: {}", e),
  };
  let ext = path.extension().unwrap_or_default().to_string_lossy();
  file.dirty = true;
  match file.metadata {
    Metadata::Id3(ref mut tag) => {
      let mut pic_frames: Vec<_> = tag
        .frames()
        .filter(|frame| frame.content().picture().is_some())
        .map(|frame| frame.clone())
        .collect();
      let pic_frame = pic_frames.get_mut(index).unwrap();
      let pic = pic_frame.content().picture().unwrap();
      let new_pic = id3::frame::Picture {
        mime_type: match ext.as_ref() {
          "jpg" | "jpeg" => "image/jpeg".to_string(),
          "png" => "image/png".to_string(),
          ext => throw!("Unsupported file type: {}", ext),
        },
        picture_type: pic.picture_type,
        description: pic.description.clone(),
        data: new_bytes,
      };
      let new_frame = id3::Frame::with_content(pic_frame.id(), id3::Content::Picture(new_pic));
      *pic_frame = new_frame;
      tag.remove_all_pictures();
      for pic_frame in pic_frames {
        tag.add_frame(pic_frame);
      }
    }
    Metadata::Mp4(ref mut tag) => {
      let mut artworks: Vec<_> = tag.take_artworks().collect();
      let artwork = artworks.get_mut(index).unwrap();
      *artwork = mp4ameta::Img {
        fmt: match ext.as_ref() {
          "jpg" | "jpeg" => mp4ameta::ImgFmt::Jpeg,
          "png" => mp4ameta::ImgFmt::Png,
          "bmp" => mp4ameta::ImgFmt::Bmp,
          ext => throw!("Unsupported file type: {}", ext),
        },
        data: new_bytes,
      };
      tag.set_artworks(artworks);
    }
  }
  Ok(())
}
