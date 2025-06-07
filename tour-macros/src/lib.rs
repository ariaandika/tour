//! Macros for [`tour`][1] template
//!
//! # Crate Structure
//!
//! ## Common
//!
//! - [`syntax`], template syntax definition
//! - [`common`], shared common behavior
//!
//! ## Inputs
//!
//! - [`config`], deserialize shared configuration file
//! - [`attribute`], deserialize derive macro attribute
//! - [`visitor`], deserialize source code
//!
//! ## Data
//!
//! - [`Metadata`], template identity
//! - [`File`], template source code data
//! - [`Template`], `Metadata` and `File`
//!
//! ## Codegen
//!
//! - [`codegen`], generate template body
//! - [`sizehint`], generate size hint calculation
//! - [`template`], combine all code generation
//!
//! [1]: <https://docs.rs/tour>
//! [`Metadata`]: data::Metadata
//! [`File`]: data::File
//! [`Template`]: data::Template

mod syntax;
mod common;

// ===== Input =====
mod config;
mod attribute;
mod visitor;

// ===== Data =====
mod data;

// ===== Output =====
mod codegen;
mod sizehint;
mod template;


/// Derive macro for `Template` trait
#[proc_macro_derive(Template, attributes(template))]
pub fn template(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match template::template(syn::parse_macro_input!(input as syn::DeriveInput)) {
        Ok(ok) => ok.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

