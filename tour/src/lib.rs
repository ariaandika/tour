//! # Tour Template
//!
//! Tour Template is a compile-time templating library with support for runtime reload.
//!
//! # Runtime Reload
//!
//! Given type with [`Template`] derive macro:
//!
//! ```ignore
//! // main.rs
//! use tour::Template;
//!
//! #[derive(Template)]
//! #[template(path = "index.html")]
//! struct Tasks {
//!     tasks: Vec<String>,
//! }
//!
//! let result = Tasks { tasks: vec![] }.render().unwrap();
//! println!("{result}");
//! ```
//!
//! Templates are searched from `templates` directory from project root by default, so above
//! example will search for `templates/index.html`.
//!
//! ```html
//! <!-- templates/index.html -->
//! {{ for task in tasks }}
//!     Task: {{ task.get(1..6) }}
//! {{ else }}
//!     No Tasks
//! {{ endfor }}
//! ```
//!
//! In debug mode, changing non expression like `No Tasks` in the source file, will
//! change the output with the new content on the next render without recompiling.
//!
//! Note that changing expression like `{{ for task in tasks }}` still requires recompile. An
//! attempt to render it without recompile, will change nothing and may result in error.
//!
//! This is still better than require to recompile on every small changes. In practice, quick
//! changes iteration is used for style changes.
//!
//! [`Template`]: tour_macros::Template
mod template;
mod write;
mod display;
mod error;

pub use template::Template;
pub use write::{TemplWrite, Escape, FmtTemplWrite, IoTemplWrite, TemplWriteFmt};
pub use display::{TemplDisplay, Display, Debug};
pub use error::{Error, Result};

#[doc(no_inline)]
pub use tour_core::{Parser, StaticVisitor};
#[doc(no_inline)]
pub use tour_macros::Template;
