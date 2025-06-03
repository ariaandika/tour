use std::{fmt, io};
use tour_core::ParseError;

/// [`Result`][std::result::Result] alias for [`Error`].
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// An error that can occur during rendering.
#[derive(Debug)]
pub enum Error {
    Parse(ParseError),
    Io(io::Error),
    NoBlock,
}

impl Error {
    /// Convert error to [`io::Error`].
    ///
    /// [`ParseError`] will become [`io::ErrorKind::InvalidData`].
    pub fn into_io(self) -> io::Error {
        match self {
            Self::Parse(err) => io::Error::new(io::ErrorKind::InvalidData, err),
            Self::Io(error) => error,
            Self::NoBlock => io::Error::new(io::ErrorKind::NotFound, "no such block"),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Parse(error) => error.fmt(f),
            Self::Io(error) => error.fmt(f),
            Self::NoBlock => f.write_str("no such block")
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

