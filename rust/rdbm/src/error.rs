//! The crate's error type.

use std::fmt;
use std::io;

#[derive(Debug)]
pub enum Error {
    /// The operating system reported an I/O failure.
    Io(io::Error),
    /// The file is not a database, or its contents violate the format.
    Corrupt(String),
    /// A key and value together are too large to fit in one page.
    TooLarge { size: usize, max: usize },
    /// A creation option is out of range.
    InvalidOption(String),
    /// An earlier mutation through this handle failed part way, so the file
    /// may hold some of its writes. Further mutations are refused.
    Poisoned,
}

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "{e}"),
            Error::Corrupt(msg) => write!(f, "corrupt database: {msg}"),
            Error::TooLarge { size, max } => write!(
                f,
                "key and value total {size} bytes; at most {max} fit in one page"
            ),
            Error::InvalidOption(msg) => write!(f, "invalid option: {msg}"),
            Error::Poisoned => write!(
                f,
                "an earlier write failed and may have left the file inconsistent; \
                 reopen it and run `rdbm check`"
            ),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}
