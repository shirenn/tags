use serde::{Deserialize, Serialize};
use std::fmt;
use std::path;

pub struct AudioFile {
    file: taglib::File,
}

#[derive(Serialize, Deserialize)]
pub struct AudioTags {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genre: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track: Option<u32>,
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
        macro_rules! show_tag {
			($name: expr, $field:tt, $( $ref:tt )*) => {{
				if let Some($( $ref )* tag) = self.$field {
                    writeln!(f, "{}:\t{}", $name, tag)?
				}
			}};
			($name:expr, $field:tt) => { show_tag!($name, $field,) };
		}
        show_tag!("title", title, ref);
        show_tag!("artist", artist, ref);
        show_tag!("album", album, ref);
        show_tag!("comment", comment, ref);
        show_tag!("genre", genre, ref);
        show_tag!("year", year);
        show_tag!("track", track);
        Ok(())
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
    pub fn update_tags(&self, tags: &AudioTags) -> Result<bool, FileError> {
        use taglib::Tag;
        let mut tag_updater = self.file.tag()?;
        let mut updated = false;
        let current_tags = self.get_tags()?;
        macro_rules! update_tag {
			($name:tt, $setter:expr, $( $ref:tt )*) => {{
				if let Some($( $ref )* tag) = tags.$name {
					if tags.$name != current_tags.$name {
						($setter)(&mut tag_updater, tag);
                        updated = true;
					}
				}
			}};
			($name:tt, $setter:expr) => { update_tag!($name, $setter,) };
		}
        update_tag!(title, Tag::set_title, ref);
        update_tag!(artist, Tag::set_artist, ref);
        update_tag!(album, Tag::set_album, ref);
        update_tag!(comment, Tag::set_comment, ref);
        update_tag!(genre, Tag::set_genre, ref);
        update_tag!(year, Tag::set_year);
        update_tag!(track, Tag::set_track);
        if updated {
            Ok(self.file.save())
        } else {
            Ok(true)
        }
    }
}
