//! tour template parser
pub mod token;
pub mod parser;
pub mod parser_v2;

pub use parser::Template;
pub use parser_v2::{Parser, /* Template,  */NoopParser, Result, Error};
