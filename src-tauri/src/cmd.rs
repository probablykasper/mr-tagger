use crate::frames::{get_frames, Frame, Metadata};
use crate::throw;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
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
pub fn error_popup(msg: String) {
  println!("Error popup: {}", msg);
  dialog::message("Error", msg);
}

#[derive(Serialize)]
pub struct Info {
  path: PathBuf,
  frames: Vec<Frame>,
}

#[command]
// pub async fn open(path: PathBuf, app: State<'_, Data>, win: Window) -> Result<(), String> {
pub async fn open_dialog(app: State<'_, Data>) -> Result<Option<Info>, String> {
  let path = dialog::FileDialogBuilder::default()
    .add_filter("Audio file", &["mp3", "m4a", "wav", "aiff"])
    .pick_file();
  let path = match path {
    None => return Ok(None),
    Some(p) => p,
  };
  return open(path, app).await;
}

#[command]
pub async fn open(path: PathBuf, app: State<'_, Data>) -> Result<Option<Info>, String> {
  let mut app = app.0.lock().unwrap();
  let ext = path.extension().unwrap_or_default().to_string_lossy();
  let metadata = match ext.as_ref() {
    "mp3" | "aiff" | "wav" => {
      let tag = match id3::Tag::read_from_path(&path) {
        Ok(tag) => tag,
        Err(_) => id3::Tag::new(),
      };
      for frame in tag.frames() {
        println!("MP3 FRAME {:?}", frame);
      }
      Metadata::Id3(tag)
    }
    "m4a" | "mp4" | "m4p" | "m4b" | "m4r" | "m4v" => {
      let tag = match mp4ameta::Tag::read_from_path(&path) {
        Ok(tag) => tag,
        Err(_) => throw!("No tags found"),
      };
      for (ident, data) in tag.data() {
        println!("M4A FRAME {:?}, {:?}", ident, data);
      }
      Metadata::Mp4(tag)
    }
    _ => throw!("Unsupported file type"),
  };
  app.open_item = Some(Item {
    path: path.clone(),
    metadata: metadata.clone(),
  });
  let info = Info {
    path: path,
    frames: get_frames(&metadata),
  };
  Ok(Some(info))
}
