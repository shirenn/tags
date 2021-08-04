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
        if let Some(ref title) = tags.title {
            if tags.title != current_tags.title {
                tag_updater.set_title(title);
            }
        }
        if let Some(ref artist) = tags.artist {
            if tags.artist != current_tags.artist {
                tag_updater.set_artist(artist);
            }
        }
        if let Some(ref album) = tags.album {
            if tags.album != current_tags.album {
                tag_updater.set_album(album);
            }
        }
        if let Some(ref comment) = tags.comment {
            if tags.comment != current_tags.comment {
                tag_updater.set_comment(comment);
            }
        }
        if let Some(ref genre) = tags.genre {
            if tags.genre != current_tags.genre {
                tag_updater.set_genre(genre);
            }
        }
        if let Some(year) = tags.year {
            if tags.year != current_tags.year {
                tag_updater.set_year(year);
            }
        }
        if let Some(track) = tags.track {
            if tags.track != current_tags.track {
                tag_updater.set_track(track);
            }
        }
        self.file.save();
        Ok(())
    }
}
