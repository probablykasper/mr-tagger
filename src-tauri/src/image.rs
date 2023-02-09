use crate::cmd::AppArg;
use crate::frames::Metadata;
use crate::throw;
use base64;
use id3::TagLike;
use lofty::ogg::OggPictureStorage;
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
pub fn get_image(index: Option<usize>, app: AppArg<'_>) -> Result<Option<Image>, String> {
  let mut app = app.0.lock().unwrap();
  let file = app.current_file()?;
  let index = match index {
    Some(index) => index,
    None => match file.metadata {
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
      Metadata::VorbisComments(ref tag) => {
        if tag.pictures().len() == 0 {
          return Ok(None);
        }
        0
      }
    },
  };
  let image_option = match file.metadata {
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
    Metadata::VorbisComments(ref tag) => match tag.pictures().get(index) {
      Some((pic, _info)) => Some(Image {
        index,
        total_images: tag.pictures().len(),
        data: base64::encode(pic.data()),
        mime_type: match pic.mime_type() {
          lofty::MimeType::Png => "image/png".to_string(),
          lofty::MimeType::Jpeg => "image/jpeg".to_string(),
          lofty::MimeType::Tiff => "image/tiff".to_string(),
          lofty::MimeType::Bmp => "image/bmp".to_string(),
          lofty::MimeType::Gif => "image/gif".to_string(),
          lofty::MimeType::Unknown(unknown) => throw!("Unknown picture type {unknown}"),
          lofty::MimeType::None => throw!("No picture type"),
          _ => throw!("Unsupported picture type"),
        },
        description: pic.description().map(|s| s.to_string()),
        picture_type: Some(match pic.pic_type() {
          lofty::PictureType::Other => "Other".to_string(),
          lofty::PictureType::Icon => "Icon".to_string(),
          lofty::PictureType::OtherIcon => "Other icon".to_string(),
          lofty::PictureType::CoverFront => "Front cover".to_string(),
          lofty::PictureType::CoverBack => "Back cover".to_string(),
          lofty::PictureType::Leaflet => "Leaflet".to_string(),
          lofty::PictureType::Media => "Media".to_string(),
          lofty::PictureType::LeadArtist => "Lead artist".to_string(),
          lofty::PictureType::Artist => "Artist".to_string(),
          lofty::PictureType::Conductor => "Conductor".to_string(),
          lofty::PictureType::Band => "Band".to_string(),
          lofty::PictureType::Composer => "Composer".to_string(),
          lofty::PictureType::Lyricist => "Lyricist".to_string(),
          lofty::PictureType::RecordingLocation => "Recording location".to_string(),
          lofty::PictureType::DuringRecording => "During recording".to_string(),
          lofty::PictureType::DuringPerformance => "During performance".to_string(),
          lofty::PictureType::ScreenCapture => "Screen capture".to_string(),
          lofty::PictureType::BrightFish => "Bright fish".to_string(),
          lofty::PictureType::Illustration => "Illustration".to_string(),
          lofty::PictureType::BandLogo => "Band logo".to_string(),
          lofty::PictureType::PublisherLogo => "Publisher logo".to_string(),
          lofty::PictureType::Undefined(u) => throw!("Undefined type {u}"),
          _ => throw!("Unsupported picture type {:?}", pic.pic_type()),
        }),
      }),
      None => None,
    },
  };
  Ok(image_option)
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
    Metadata::VorbisComments(ref mut tag) => {
      tag.remove_picture(index);
    }
  }
  file.dirty = true;
  Ok(())
}

enum MimeType {
  Png,
  Jpeg,
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
  let mime_type = match ext.as_ref() {
    "jpg" | "jpeg" => MimeType::Jpeg,
    "png" => MimeType::Png,
    ext => throw!("Unsupported file type: {}", ext),
  };
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
    Metadata::VorbisComments(ref mut tag) => {
      if index <= tag.pictures().len() {
        let info_result = match mime_type {
          MimeType::Png => lofty::PictureInformation::from_png(&new_bytes),
          MimeType::Jpeg => lofty::PictureInformation::from_jpeg(&new_bytes),
        };
        let info = match info_result {
          Ok(info) => info,
          Err(e) => throw!("Error reading picture info: {}", e),
        };
        let mut byte_cursor = std::io::Cursor::new(new_bytes);
        let pic = match lofty::Picture::from_reader(&mut byte_cursor) {
          Ok(mut pic) => {
            pic.set_pic_type(lofty::PictureType::Other);
            pic
          }
          Err(e) => throw!("Error reading picture: {}", e),
        };
        // this is safe because set_picture appends if out of bounds:
        tag.set_picture(index, pic, info);
      } else {
        throw!("Index out of range");
      }
    }
  }
  file.dirty = true;
  Ok(())
}
