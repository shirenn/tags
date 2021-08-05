use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::process::{Command, ExitStatus};

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    NonZeroExitStatus(ExitStatus),
    PathError,
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IoError(error)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::IoError(error) => write!(f, "{}", error),
            Error::NonZeroExitStatus(status) => write!(
                f,
                "Editor exited with {} exit code",
                status.code().map_or("no".to_string(), |x| x.to_string())
            ),
            Error::PathError => write!(f, "Fail to get tempfile path"),
        }
    }
}

/// Edits `content` in an external editor `editor` and returns the edited
/// content
///
/// # Errors
/// Relays errors it got from calling tempfile
/// Returns `PathError` if the path provided by tempfile is invalid
/// Returns any `NonZeroExitStatus` with the exit status value if the editor
/// did not exit with success
/// Returns an `IOError` if `edit_content` fails to read the file
pub fn edit_content(editor: &str, content: &str) -> Result<String, Error> {
    let mut file = tempfile::Builder::new()
        .prefix("tags-")
        .rand_bytes(10)
        .tempfile()?;
    file.write_all(content.as_bytes())?;

    let path = file.into_temp_path();
    let unw_path = path.to_str().ok_or(Error::PathError)?;

    let status = Command::new(&editor).arg(unw_path).status()?;

    if !status.success() {
        return Err(Error::NonZeroExitStatus(status));
    }

    fs::read_to_string(path).map_err(Error::from)
}
