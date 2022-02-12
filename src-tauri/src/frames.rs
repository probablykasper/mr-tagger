use serde::Serialize;

#[derive(Debug, Clone)]
pub enum Metadata {
  Id3(id3::Tag),
  Mp4(mp4ameta::Tag),
}
use Metadata::{Id3, Mp4};

#[derive(Serialize)]
pub enum Frame {
  Text { id: String, value: String },
}

fn get_id3_frames(tag: &id3::Tag) -> Vec<Frame> {
  let mut frames = Vec::new();
  for frame in tag.frames() {
    match frame.content() {
      id3::Content::Text(s) => {
        frames.push(Frame::Text {
          id: frame.id().to_string(),
          value: s.to_string(),
        });
      }
      // !TODO
      id3::Content::ExtendedText(_) => {}
      // !TODO
      id3::Content::Link(_) => {}
      // !TODO
      id3::Content::ExtendedLink(_) => {}
      // !TODO
      id3::Content::Comment(_) => {}
      // !TODO
      id3::Content::Lyrics(_) => {}
      // !TODO
      id3::Content::SynchronisedLyrics(_) => {}
      // !TODO
      id3::Content::Picture(_) => {}
      // !TODO
      id3::Content::EncapsulatedObject(_) => {}
      // !TODO
      id3::Content::Unknown(_) => {}
      // !TODO
      id3::Content::Popularimeter(_) => {}
      // !TODO
      id3::Content::Chapter(_) => {}
      // !TODO
      id3::Content::MpegLocationLookupTable(_) => {}

      _ => {}
    }
  }
  frames
}

fn get_mp4_frames(tag: &mp4ameta::Tag) -> Vec<Frame> {
  let mut frames = Vec::new();
  for (id, data) in tag.data() {
    match data {
      // !TODO
      mp4ameta::Data::Reserved(_) => {}
      mp4ameta::Data::Utf8(s) => {
        frames.push(Frame::Text {
          id: id.to_string(),
          value: s.to_string(),
        });
      }
      // !TODO
      mp4ameta::Data::Utf16(_) => {}
      // !TODO
      mp4ameta::Data::Jpeg(_) => {}
      // !TODO
      mp4ameta::Data::Png(_) => {}
      // !TODO
      mp4ameta::Data::BeSigned(_) => {}
      // !TODO
      mp4ameta::Data::Bmp(_) => {}
    }
  }
  frames
}

pub fn get_frames(md: &Metadata) -> Vec<Frame> {
  match md {
    Id3(id3) => get_id3_frames(id3),
    Mp4(mp4) => get_mp4_frames(mp4),
  }
}
