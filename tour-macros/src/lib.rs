//! Macros for [`tour`][1] template
//!
//! [1]: <https://docs.rs/tour>

mod syntax;
mod shared;

mod attribute;
mod parser;
mod template;

/// Derive macro for `Template` trait
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

macro_rules! error {
    (@ $s:expr, $($tt:tt)*) => {
        return Err(Error::new($s, format!($($tt)*)))
    };
    (!$s:expr, $($tt:tt)*) => {
        match $s { Some(ok) => ok, None => error!($($tt)*), }
    };
    (!$s:expr) => {
        match $s { Ok(ok) => ok, Err(err) => error!("{err}"), }
    };
    ($s:expr, $($tt:tt)*) => {
        error!(@ $s.span(), $($tt)*)
    };
    ($($tt:tt)*) => {
        error!(@ proc_macro2::Span::call_site(), $($tt)*)
    };
}

pub(crate) use error;
