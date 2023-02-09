use crate::frames::Metadata;
use crate::throw;
use id3::TagLike;
use lofty::Accessor;
use serde::Serialize;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::api::dialog;
use tauri::{command, State};

#[derive(Clone, Serialize)]
pub struct File {
  pub path: PathBuf,
  pub dirty: bool,
  #[serde(skip_serializing)]
  pub metadata: Metadata,
}

#[derive(Default, Serialize)]
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

#[command]
pub fn close_window(win: tauri::Window) {
  let _ = win.close();
}

fn id3_split<'a>(s: Option<&'a str>) -> Vec<&'a str> {
  match s {
    Some(s) => s.split('\u{0}').collect(),
    None => vec![],
  }
}

fn get_frame_text<'a>(tag: &'a id3::Tag, id: &str) -> Option<&'a str> {
  let frame = tag.get(id)?;
  let text = frame.content().text()?;
  return Some(text);
}

fn opt_to_str<'a>(n: Option<impl ToString>) -> String {
  match n {
    Some(n) => n.to_string(),
    None => "".to_string(),
  }
}

#[command]
pub fn get_page(app: AppArg<'_>) -> Option<Value> {
  let mut app = app.0.lock().unwrap();
  let file = app.current_file().ok()?;

  let title = match file.metadata {
    Metadata::Id3(ref tag) => tag.title().unwrap_or("").to_string(),
    Metadata::Mp4(ref tag) => tag.title().unwrap_or("").to_string(),
    Metadata::VorbisComments(ref tag) => tag.title().map(|s| s.to_string()).unwrap_or_default(),
  };

  let artists: Vec<_> = match file.metadata {
    Metadata::Id3(ref tag) => id3_split(tag.artist()),
    Metadata::Mp4(ref tag) => tag.artists().collect(),
    Metadata::VorbisComments(ref tag) => tag.get_all("ARTIST").collect(),
  };

  let album = match file.metadata {
    Metadata::Id3(ref tag) => tag.album().unwrap_or("").to_string(),
    Metadata::Mp4(ref tag) => tag.album().unwrap_or("").to_string(),
    Metadata::VorbisComments(ref tag) => tag.album().map(|s| s.to_string()).unwrap_or_default(),
  };

  let album_artists: Vec<_> = match file.metadata {
    Metadata::Id3(ref tag) => id3_split(tag.album_artist()),
    Metadata::Mp4(ref tag) => tag.album_artists().collect(),
    Metadata::VorbisComments(ref tag) => tag.get_all("ALBUMARTIST").collect(),
  };

  let composer = match file.metadata {
    Metadata::Id3(ref tag) => id3_split(get_frame_text(&tag, "TCOM")),
    Metadata::Mp4(ref tag) => tag.composers().collect(),
    Metadata::VorbisComments(ref tag) => tag.get_all("COMPOSER").collect(),
  };

  let groupings = match file.metadata {
    // non-standard iTunes tag, three-byte ID3v2.2 tag is not remapped
    Metadata::Id3(ref tag) => id3_split(match get_frame_text(&tag, "GRP1") {
      Some(s) => Some(s),
      None => get_frame_text(&tag, "GP1"),
    }),
    Metadata::Mp4(ref tag) => tag.groupings().collect(),
    Metadata::VorbisComments(ref tag) => tag.get_all("GROUPING").collect(),
  };

  let genres = match file.metadata {
    Metadata::Id3(ref tag) => id3_split(tag.genre()),
    Metadata::Mp4(ref tag) => tag.genres().collect(),
    Metadata::VorbisComments(ref tag) => tag.get_all("GENRE").collect(),
  };

  let track_num = match file.metadata {
    Metadata::Id3(ref tag) => opt_to_str(tag.track()),
    Metadata::Mp4(ref tag) => opt_to_str(tag.track().0),
    Metadata::VorbisComments(ref tag) => opt_to_str(tag.track()),
  };
  let track_total = match file.metadata {
    Metadata::Id3(ref tag) => opt_to_str(tag.total_tracks()),
    Metadata::Mp4(ref tag) => opt_to_str(tag.track().1),
    Metadata::VorbisComments(ref tag) => opt_to_str(tag.track_total()),
  };

  let disc_num = match file.metadata {
    Metadata::Id3(ref tag) => opt_to_str(tag.disc()),
    Metadata::Mp4(ref tag) => opt_to_str(tag.disc().0),
    Metadata::VorbisComments(ref tag) => opt_to_str(tag.disk()),
  };
  let disc_total = match file.metadata {
    Metadata::Id3(ref tag) => opt_to_str(tag.total_discs()),
    Metadata::Mp4(ref tag) => opt_to_str(tag.disc().1),
    Metadata::VorbisComments(ref tag) => opt_to_str(tag.disk_total()),
  };

  let compilation = match file.metadata {
    Metadata::Id3(ref tag) => {
      // non-standard iTunes tag, three-byte ID3v2.2 tag is not remapped
      get_frame_text(&tag, "TCMP") == Some("1") || get_frame_text(&tag, "TCP") == Some("1")
    }
    Metadata::Mp4(ref tag) => tag.compilation(),
    Metadata::VorbisComments(ref tag) => tag.get("COMPILATION") == Some("1"),
  };

  let bpm = match file.metadata {
    Metadata::Id3(ref tag) => get_frame_text(&tag, "TBPM").unwrap_or("").to_string(),
    Metadata::Mp4(ref tag) => opt_to_str(tag.bpm()),
    Metadata::VorbisComments(ref tag) => tag.get("BPM").unwrap_or("").to_string(),
  };

  #[derive(Serialize)]
  struct Comment {
    text: String,
    lang: Option<String>,
    description: Option<String>,
  }
  let comments: Vec<Comment> = match file.metadata {
    Metadata::Id3(ref tag) => tag
      .comments()
      .map(|c| Comment {
        text: c.text.clone(),
        lang: Some(c.lang.clone()),
        description: Some(c.description.clone()),
      })
      .collect(),
    Metadata::Mp4(ref tag) => tag
      .comments()
      .map(|c| Comment {
        text: c.to_string(),
        lang: None,
        description: None,
      })
      .collect(),
    Metadata::VorbisComments(ref tag) => tag
      .get_all("COMMENT")
      .map(|c| Comment {
        text: c.to_string(),
        lang: None,
        description: None,
      })
      .collect(),
  };

  Some(serde_json::json!({
    "path": file.path.clone(),
    "title": title,
    "artists": artists,
    "album": album,
    "album_artists": album_artists,
    "composer": composer,
    "groupings": groupings,
    "genres": genres,
    // year
    "track_num": track_num,
    "track_total": track_total,
    "disc_num": disc_num,
    "disc_total": disc_total,
    "compilation": compilation,
    "bpm": bpm,
    "comments": comments,
    "frames": file.metadata.get_frames(),
  }))
}
