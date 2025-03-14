//! macros for tour template
use proc_macro::TokenStream;

mod template;

/// derive macro for `Template` trait
#[proc_macro_derive(Template, attributes(template))]
pub fn template(input: TokenStream) -> TokenStream {
    match template::template(syn::parse_macro_input!(input as syn::DeriveInput)) {
        Ok(ok) => ok.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

