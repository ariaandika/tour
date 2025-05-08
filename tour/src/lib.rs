//! tour template
mod template;
mod write;
mod display;
mod error;

pub use template::Template;
pub use write::{TemplWrite, Escape, FmtTemplWrite, IoTemplWrite, TemplWriteFmt};
pub use display::{TemplDisplay, Display};
pub use error::{Error, Result};

pub use tour_core::{Parser, NoopParser};
pub use tour_macros::Template;
