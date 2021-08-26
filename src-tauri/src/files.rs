use crate::cmd::{AppArg, File};
use crate::frames::Metadata;
use crate::throw;
use std::fs;
use std::path::PathBuf;
use tauri::api::dialog;
use tauri::command;

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
pub async fn save_file(index: usize, save_as: bool, app: AppArg<'_>) -> Result<(), String> {
  let mut app = app.0.lock().unwrap();
  let mut file = app.files.get_mut(index).unwrap();
  if save_as {
    let ext = file.path.extension().unwrap_or_default().to_string_lossy();
    let (sender, receiver) = std::sync::mpsc::channel();
    dialog::FileDialogBuilder::new()
      .set_file_name("report.kryp")
      .add_filter("Audio/Video file", &[&ext])
      .save_file(move |p| {
        sender.send(p).unwrap();
      });
    let new_path = match receiver.recv().unwrap_or_default() {
      Some(file_path) => file_path,
      None => return Ok(()),
    };
    match fs::copy(&file.path, &new_path) {
      Ok(_) => {}
      Err(e) => throw!("Error copying file: {}", e),
    }
    file.path = new_path;
  }
  match file.metadata {
    Metadata::Id3(ref tag) => match tag.write_to_path(&file.path, id3::Version::Id3v24) {
      Ok(_) => {}
      Err(e) => throw!("Error saving file: {}", e.description),
    },
    Metadata::Mp4(ref tag) => match tag.write_to_path(&file.path) {
      Ok(_) => {}
      Err(e) => throw!("Error saving file: {}", e.description),
    },
  }
  file.dirty = false;
  Ok(())
}
