//! tour template
mod template;
mod write;
mod display;
mod error;

pub use template::Template;
pub use write::{Writer, Escape};
pub use display::Display;
pub use error::{Error, Result};

pub use tour_core::{Parser, NoopParser};
pub use tour_macros::Template;
