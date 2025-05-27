
/// [`Result`][std::result::Result] alias for [`ParseError`].
pub type Result<T,E = ParseError> = core::result::Result<T,E>;

/// An error that may occur during parsing in [`Parser`][super::Parser].
#[derive(Debug)]
pub enum ParseError {
    Generic(String),
}

impl std::error::Error for ParseError {}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Generic(s) => f.write_str(s),
        }
    }
}


