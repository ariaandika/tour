//! tour template
pub mod render;
pub mod template;

pub use render::{Display, Writer};
pub use template::Template;
pub use tour_parser::{/* Parser,  */parser_v2::{Parser, NoopParser}};
pub use tour_macros::Template;
