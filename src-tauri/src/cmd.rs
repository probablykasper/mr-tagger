use crate::frames::{get_frames, Frame, Metadata};
use crate::throw;
use serde::Serialize;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::api::dialog;
use tauri::{command, State};

#[derive(Debug, Clone, Serialize)]
pub struct File {
  pub path: PathBuf,
  pub dirty: bool,
  #[serde(skip_serializing)]
  pub metadata: Metadata,
}

#[derive(Debug, Default, Serialize)]
pub struct App {
  pub current_index: usize,
  pub files: Vec<File>,
}
impl App {
  pub fn current_file(&mut self) -> Result<&mut File, String> {
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

pub type AppArg<'a> = State<'a, AppState>;

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
