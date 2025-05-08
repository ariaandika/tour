//! tour template parser

mod spec;

mod parser;

pub use spec::Delimiter;
pub use parser::{Parser, Template, ExprParser, NoopParser, ParseError, Result};

