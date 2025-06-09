//! Macros for [`tour`] template
//!
//! # `tour::Template` Trait
//!
//! The goal is to generate `tour::Template` trait implementation.
//!
//! All code generation requires [`Template`] struct. To build it, requires [`Metadata`] and
//! [`File`].
//!
//! # [`Metadata`]
//!
//! `Metadata` contains template identity that explicitly set by user.
//!
//! `Metadata` can be represented by:
//!
//! - [`Attribute`], e.g. `#[template(path = "index.html")]`
//! - [`LayoutTempl`], e.g. `{{ layout "layout.html" }}`
//! - [`RenderTempl`], e.g. `{{ render "navbar.html" }}`
//! - [`UseTempl`], e.g. `{{ use "button.html" as Button }}`
//!
//! # [`File`]
//!
//! `File` contains the actual template content.
//!
//! `File` can be created using [`Metadata`].
//!
//! # Code generation
//!
//! For the `tour::Template` trait, code will be generated for the following methods:
//!
//! ## `render_into()`, `render_block_into()` and `contains_block()`
//!
//! `render_into()` is the main template rendering logic. Generated code includes:
//!
//! - destructured fields, for convenient
//! - `include_str!()` the template file, to trigger recompile when changed
//! - static content reparsing for runtime reload
//! - the main template content rendering
//!
//! `render_block_into()` contains the same code but for selected block only.
//!
//! Block can be rendered at runtime only when declared with the `pub` keyword.
//!
//! `contains_block()` contains basic check whether a block is available to be rendered at runtime.
//!
//! ## `size_hint()` and `size_hint_block()`
//!
//! `size_hint()` will calculate the lower and upper bounds of the rendered template length.
//!
//! `size_hint_block()` is the same but for selected block only.
//!
//! Note that currently, size hint calculation does not do any runtime check. So rendered template
//! may exceed the upper bounds. But lower bounds is pretty much accurate.
//!
//! Runtime check like an `if` branch expression that have been evaluated in struct field, can
//! produce much more accurate size hints. This will be improved in the future.
//!
//! For now, size hint is statically calculated. Its better than nothing.
//!
//! # External template
//!
//! Other external template that is referenced will also be generated.
//!
//! External template can be referenced by:
//!
//! - [`LayoutTempl`], e.g. `{{ layout "layout.html" }}`
//! - [`RenderTempl`], e.g. `{{ render "navbar.html" }}`
//! - [`UseTempl`], e.g. `{{ use "button.html" as Button }}`
//!
//! [`tour`]: <https://docs.rs/tour>
//! [`Template`]: data::Template
//! [`Metadata`]: data::Metadata
//! [`File`]: data::File
//! [`Attribute`]: syn::Attribute
//! [`LayoutTempl`]: syntax::LayoutTempl
//! [`RenderTempl`]: syntax::RenderTempl
//! [`UseTempl`]: syntax::UseTempl

pub mod syntax;
pub mod ast;
pub mod common;

// ===== Input =====
pub mod config;

// ===== Data =====
pub mod metadata;
pub mod file;
pub mod data;

// ===== Output =====
pub mod codegen;

