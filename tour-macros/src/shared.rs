use proc_macro2::TokenStream;
use quote::quote;
use tour_core::Delimiter;

/// `ToTokens` for public name
pub struct TemplDisplay;

impl quote::ToTokens for TemplDisplay {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        quote::quote! {::tour::TemplDisplay}.to_tokens(tokens);
    }
}

/// `ToTokens` for public name
pub struct TemplWrite;

impl quote::ToTokens for TemplWrite {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        quote::quote! {::tour::TemplWrite}.to_tokens(tokens);
    }
}

#[derive(Clone)]
pub enum Reload {
    Debug,
    Always,
    Never,
    Expr(syn::Expr),
}

pub enum SourceTempl {
    Path(String),
    Root(String),
    Source(String),
}

impl SourceTempl {
    pub fn resolve_source(&self) -> syn::Result<std::borrow::Cow<'_,str>> {
        match self.resolve_path() {
            Some(path) => Ok(error!(!std::fs::read_to_string(path)).into()),
            None => if let Self::Source(src) = &self {
                Ok(src.into())
            } else {
                unreachable!()
            },
        }
    }

    /// Return `Some` if template is external and have path.
    pub fn resolve_path(&self) -> Option<String> {
        match self {
            Self::Path(path) => {
                let mut cwd = std::env::current_dir().expect("failed to get current directory");
                cwd.push("templates");
                cwd.push(path);
                Some(cwd.to_string_lossy().into_owned())
            },
            Self::Root(path) => {
                let mut cwd = std::env::current_dir().expect("failed to get current directory");
                cwd.push(path);
                Some(cwd.to_string_lossy().into_owned())
            }
            Self::Source(_) => None,
        }
    }
}

pub fn display(delim: Delimiter, expr: &syn::Expr) -> TokenStream {
    use Delimiter::*;

    match delim {
        Quest => quote! {&::tour::Debug(&#expr)},
        Percent => quote! {&::tour::Display(&#expr)},
        Brace | Bang | Hash => quote! {&#expr},
    }
}

pub fn writer(delim: Delimiter) -> TokenStream {
    use Delimiter::*;

    match delim {
        Bang => quote! {&mut *writer},
        Brace | Percent | Quest | Hash => quote! {&mut ::tour::Escape(&mut *writer)},
    }
}

/// Everything will return `Result<T, syn::Error>`
///
/// `error!("{}",error)`, standard `format!`
///
/// `error!(attr, "{}", error)`, standard `format!` with `attr`s span.
///
/// `error!(!option, "{}", error)`, unwrap option with error as standard `format!`.
///
/// `error!(!result)`, unwrap result.
macro_rules! error {
    (@ $s:expr, $($tt:tt)*) => {
        return Err(syn::Error::new($s, format!($($tt)*)))
    };
    (!$s:expr, $($tt:tt)*) => {
        match $s { Some(ok) => ok, None => crate::shared::error!($($tt)*), }
    };
    (!$s:expr) => {
        match $s { Ok(ok) => ok, Err(err) => crate::shared::error!("{err}"), }
    };
    ($s:expr, $($tt:tt)*) => {
        crate::shared::error!(@ $s.span(), $($tt)*)
    };
    ($($tt:tt)*) => {
        crate::shared::error!(@ proc_macro2::Span::call_site(), $($tt)*)
    };
}

pub(crate) use error;
