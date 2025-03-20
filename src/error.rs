use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    Config(String),
    Database(String),
    Storage(String),
    Backup(String),
    Restore(String),
    Io(std::io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Config(msg) => write!(f, "Configuration error: {}", msg),
            Error::Database(msg) => write!(f, "Database error: {}", msg),
            Error::Storage(msg) => write!(f, "Storage error: {}", msg),
            Error::Backup(msg) => write!(f, "Backup error: {}", msg),
            Error::Restore(msg) => write!(f, "Restore error: {}", msg),
            Error::Io(err) => write!(f, "I/O error: {}", err),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

// Type alias for Result with our custom Error
pub type Result<T> = std::result::Result<T, Error>;