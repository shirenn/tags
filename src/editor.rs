use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::process::{Command, ExitStatus};

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

pub fn edit_content(editor: &str, content: &str) -> Result<String, Error> {
    let mut file = tempfile::Builder::new()
        .prefix("taggie-")
        .suffix(".tsv")
        .rand_bytes(5)
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
