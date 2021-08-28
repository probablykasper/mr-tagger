use crate::cmd::AppArg;
use crate::frames::Metadata;
use crate::throw;
use base64;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use tauri::command;

#[derive(Serialize)]
pub struct Image {
  index: usize,
  total_images: usize,
  data: String,
  mime_type: String,
  description: Option<String>,
  picture_type: Option<String>,
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
        description: Some(pic.description.clone()),
        picture_type: Some(pic.picture_type.to_string()),
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
        description: None,
        picture_type: None,
      }),
      None => None,
    },
  }
}

#[command]
pub fn remove_image(index: usize, app: AppArg<'_>) -> Result<(), String> {
  let mut app = app.0.lock().unwrap();
  let file = app.current_file().unwrap();
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
  file.dirty = true;
  Ok(())
}

#[command]
pub fn set_image(index: usize, path: PathBuf, app: AppArg<'_>) -> Result<(), String> {
  let mut app = app.0.lock().unwrap();
  let file = app.current_file().unwrap();
  let new_bytes = match fs::read(&path) {
    Ok(b) => b,
    Err(e) => throw!("Error reading that file: {}", e),
  };
  let ext = path.extension().unwrap_or_default().to_string_lossy();
  match file.metadata {
    Metadata::Id3(ref mut tag) => {
      let mut pic_frames: Vec<_> = tag
        .frames()
        .filter(|frame| frame.content().picture().is_some())
        .map(|frame| frame.clone())
        .collect();
      let mime_type = match ext.as_ref() {
        "jpg" | "jpeg" => "image/jpeg".to_string(),
        "png" => "image/png".to_string(),
        ext => throw!("Unsupported file type: {}", ext),
      };
      let mut new_pic = id3::frame::Picture {
        mime_type,
        picture_type: id3::frame::PictureType::Other,
        description: "".to_string(),
        data: new_bytes,
      };
      match pic_frames.get_mut(index) {
        Some(old_frame) => {
          let old_pic = old_frame.content().picture().unwrap();
          new_pic.picture_type = old_pic.picture_type;
          new_pic.description = old_pic.description.clone();
          let new_frame = id3::Frame::with_content("APIC", id3::Content::Picture(new_pic));
          *old_frame = new_frame;
        }
        None => {
          if index == pic_frames.len() {
            let new_frame = id3::Frame::with_content("APIC", id3::Content::Picture(new_pic));
            pic_frames.insert(index, new_frame);
          } else {
            throw!("Index out of range");
          }
        }
      }
      tag.remove_all_pictures();
      for pic_frame in pic_frames {
        tag.add_frame(pic_frame);
      }
    }
    Metadata::Mp4(ref mut tag) => {
      let mut artworks: Vec<_> = tag.take_artworks().collect();
      let new_artwork = mp4ameta::Img {
        fmt: match ext.as_ref() {
          "jpg" | "jpeg" => mp4ameta::ImgFmt::Jpeg,
          "png" => mp4ameta::ImgFmt::Png,
          "bmp" => mp4ameta::ImgFmt::Bmp,
          ext => throw!("Unsupported file type: {}", ext),
        },
        data: new_bytes,
      };
      match artworks.get_mut(index) {
        Some(artwork) => {
          *artwork = new_artwork;
        }
        None => {
          if index == artworks.len() {
            artworks.push(new_artwork);
          } else {
            throw!("Index out of range");
          }
        }
      }
      tag.set_artworks(artworks);
    }
  }
  file.dirty = true;
  Ok(())
}
