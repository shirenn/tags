use serde::{Deserialize, Serialize};
use std::fmt;
use std::path;


pub struct AudioFile {
    file: taglib::File,
}

#[derive(Serialize, Deserialize)]
pub struct AudioTags {
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    artist: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    album: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    genre: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    year: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    track: Option<u32>,
}

#[derive(Debug)]
pub enum FileError {
    NotAFile,
    TaglibError(taglib::FileError),
}

impl From<taglib::FileError> for FileError {
    fn from(error: taglib::FileError) -> Self {
        FileError::TaglibError(error)
    }
}

impl fmt::Display for AudioTags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}

impl AudioFile {
    pub fn new(path: &path::Path) -> Result<Self, FileError> {
        if !path.is_file() {
            return Err(FileError::NotAFile);
        }
        let file = taglib::File::new(&path)?;
        Ok(AudioFile { file })
    }
    pub fn get_tags(&self) -> Result<AudioTags, FileError> {
        let tag = self.file.tag()?;
        Ok(AudioTags {
            title: tag.title(),
            artist: tag.artist(),
            album: tag.album(),
            comment: tag.comment(),
            genre: tag.genre(),
            year: tag.year(),
            track: tag.track(),
        })
    }
    pub fn apply_tags(&self, tags: &AudioTags) -> Result<(), FileError> {
        let mut tag_updater = self.file.tag()?;
        let current_tags = self.get_tags()?;
        macro_rules! update_tag {
            ($name:tt, $setter:expr, $( $ref:tt )*) => {{
                if let Some($( $ref )* tag) = tags.$name {
                    if tags.$name != current_tags.$name {
                        ($setter)(&mut tag_updater, tag)
                    }
                }
            }};
            ($name:tt, $setter:expr) => { update_tag!($name, $setter,) };
        }
        use taglib::Tag;
        update_tag!(title, Tag::set_title, ref);
        update_tag!(artist, Tag::set_artist, ref);
        update_tag!(album, Tag::set_album, ref);
        update_tag!(comment, Tag::set_comment, ref);
        update_tag!(genre, Tag::set_genre, ref);
        update_tag!(year, Tag::set_year);
        update_tag!(track, Tag::set_track);
        self.file.save();
        Ok(())
    }
}
