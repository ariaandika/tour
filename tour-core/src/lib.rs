//! The [`tour`][1] template parser.
//!
//! The [`Parser`] type will only scan for delimiters to split expressions and static contents.
//! [`Parser`] requires a [`Visitor`] implementation which will actually process the inputs.
//!
//! For example:
//!
//! ```html
//! Hello {{ name.to_uppercase() }}
//! ```
//!
//! [`Parser`] will split the input, and call [`Visitor::visit_static`] with `"Hello "`, and
//! [`Visitor::visit_expr`] with `"name.to_uppercase()"`.
//!
//! This separation allows for both compile time and runtime template loading without bringing an
//! entire parser in the binary.
//!
//! The [`tour-macros`][2] contains implementation of [`Visitor`] utilizing the [`syn`][1]
//! crate, which allows rust expression inside template.
//!
//! There is also [`StaticVisitor`] that only collect static content. This implementations is used
//! in runtime template reloading.
//!
//! [1]: <https://docs.rs/tour>
//! [2]: <https://docs.rs/tour-macros>
//! [3]: <https://docs.rs/syn>
mod syntax;
mod visitor;
mod parser;
mod error;

pub use syntax::Delimiter;
pub use visitor::{Visitor, StaticVisitor};
pub use parser::Parser;
pub use error::{Result, ParseError};
