//! Macros for [`tour`][1] template
//!
//! [1]: <https://docs.rs/tour>

mod syntax;
mod shared;

mod attribute;
mod visitor;

mod sizehint;
mod codegen;
mod template;

/// Derive macro for `Template` trait
#[proc_macro_derive(Template, attributes(template,fmt))]
pub fn template(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match template::template(syn::parse_macro_input!(input as syn::DeriveInput)) {
        Ok(ok) => ok.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

