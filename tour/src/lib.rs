//! tour template
mod template;
mod write;
mod display;
mod error;

pub use template::Template;
pub use write::{TemplWrite, Escape, FmtTemplWrite, IoTemplWrite, TemplWriteFmt};
pub use display::{TemplDisplay, Display, Debug};
pub use error::{Error, Result};

pub use tour_core::{Parser, StaticVisitor};
pub use tour_macros::Template;
