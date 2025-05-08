//! macros for `tour` template
mod syntax;
mod parser;
mod template;

/// derive macro for `Template` trait
#[proc_macro_derive(Template, attributes(template,fmt))]
pub fn template(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match template::template(syn::parse_macro_input!(input as syn::DeriveInput)) {
        Ok(ok) => ok.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

/// `ToTokens` for public name
struct TemplDisplay;

impl quote::ToTokens for TemplDisplay {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        quote::quote! {::tour::TemplDisplay}.to_tokens(tokens);
    }
}

/// `ToTokens` for public name
struct TemplWrite;

impl quote::ToTokens for TemplWrite {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        quote::quote! {::tour::TemplWrite}.to_tokens(tokens);
    }
}

