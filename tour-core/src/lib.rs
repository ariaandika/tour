//! The [`tour`][1] template parser.
//!
//! The [`Parser`] type will only parse delimiters to split expressions and static contents.
//! [`Parser`] requires an [`ExprParser`] implementation which will parse the expressions.
//!
//! ```html
//! Hello {{ name.to_uppercase() }}
//! ```
//!
//! [`Parser`] will store `"Hello "` and pass `"name.to_uppercase()"` to [`ExprParser`].
//!
//! This separation allows for both compile time and runtime template loading without bringing an
//! entire parser in the binary.
//!
//! The [`tour-macros`][2] contains implementation of [`ExprParser`] utilizing the [`syn`][1]
//! crate, which allows rust expression inside template.
//!
//! There is also [`NoopParser`] implementation that does nothing to the expression. This
//! implementations is used in runtime template reloading.
//!
//! [1]: <https://docs.rs/tour>
//! [2]: <https://docs.rs/tour-macros>
//! [3]: <https://docs.rs/syn>
mod syntax;
mod parser;

pub use syntax::Delimiter;
pub use parser::{Parser, Template, ExprParser, NoopParser, ParseError, Result};

