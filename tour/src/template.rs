use crate::Writer;

pub trait Template {
    fn render_into(&self, render: &mut impl Writer) -> Result<()>;

    fn render_layout_into(&self, render: &mut impl Writer) -> Result<()> {
        self.render_into(render)
    }

    fn render(&self) -> Result<String> {
        let mut buffer = String::with_capacity(128);
        self.render_into(&mut buffer)?;
        Ok(buffer)
    }

    fn render_layout(&self) -> Result<String> {
        let mut buffer = String::with_capacity(128);
        self.render_layout_into(&mut buffer)?;
        Ok(buffer)
    }
}

pub type Result<T,E = Error> = std::result::Result<T,E>;

#[derive(Debug)]
pub enum Error {
    Parse(tour_parser::parser::Error),
    Io(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Parse(error) => error.fmt(f),
            Error::Io(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

impl From<tour_parser::parser::Error> for Error {
    fn from(value: tour_parser::parser::Error) -> Self {
        Self::Parse(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}


