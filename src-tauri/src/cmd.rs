use serde::Serialize;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::api::dialog;
use tauri::{command, State};

use crate::throw;

#[derive(Debug, Clone)]
enum Metadata {
  Id3(id3::Tag),
  M4a(mp4ameta::Tag),
}
use Metadata::{Id3, M4a};

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

#[derive(Serialize)]
pub struct Info {
  path: PathBuf,
  title: String,
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
      Metadata::Id3(tag)
    }
    "m4a" | "mp4" | "m4p" | "m4b" | "m4r" | "m4v" => {
      let tag = match mp4ameta::Tag::read_from_path(&path) {
        Ok(tag) => tag,
        Err(_) => throw!("No tags found"),
      };
      Metadata::M4a(tag)
    }
    _ => throw!("Unsupported file type"),
  };
  app.open_item = Some(Item {
    path: path.clone(),
    metadata: metadata.clone(),
  });
  let info = Info {
    path: path,
    title: match metadata {
      Id3(tag) => tag.title().unwrap_or_default().to_string(),
      M4a(tag) => tag.title().unwrap_or_default().to_string(),
    },
  };
  Ok(Some(info))
}
