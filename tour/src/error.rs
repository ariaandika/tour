use std::{fmt, io};
use tour_core::ParseError;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub enum Error {
    Parse(ParseError),
    Io(io::Error),
}

impl Error {
    /// Convert error to [`io::Error`].
    ///
    /// [`ParseError`] will become [`io::ErrorKind::InvalidData`].
    pub fn into_io(self) -> io::Error {
        match self {
            Error::Parse(err) => io::Error::new(io::ErrorKind::InvalidData, err),
            Error::Io(error) => error,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Parse(error) => error.fmt(f),
            Error::Io(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

impl From<ParseError> for Error {
    fn from(value: ParseError) -> Self {
        Self::Parse(value)
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<fmt::Error> for Error {
    fn from(_: fmt::Error) -> Self {
        Self::Io(io::ErrorKind::Other.into())
    }
}

