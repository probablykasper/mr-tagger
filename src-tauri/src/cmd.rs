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
