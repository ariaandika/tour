//! macros for `tour` template
mod syntax;
mod parser;
mod template;

/// derive macro for `Template` trait
#[proc_macro_derive(Template, attributes(template))]
pub fn template(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match template::template(syn::parse_macro_input!(input as syn::DeriveInput)) {
        Ok(ok) => ok.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

