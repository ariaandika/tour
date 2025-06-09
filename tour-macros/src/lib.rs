//! Macros for `tour` library.
//!
//! [`tour`]: <https://docs.rs/tour>

/// Derive macro for `Template` trait
#[proc_macro_derive(Template, attributes(template))]
pub fn template(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match tour_parser::codegen::generate(&syn::parse_macro_input!(input as syn::DeriveInput)) {
        Ok(ok) => ok.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

