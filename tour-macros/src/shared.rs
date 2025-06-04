use proc_macro2::TokenStream;
use quote::quote;
use std::borrow::Cow;
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

/// Runtime template reload behavior.
#[derive(Clone)]
pub enum Reload {
    Debug,
    Always,
    Never,
    Expr(Box<syn::Expr>),
}

/// Template reference.
///
/// Used in derive attribute (`#[path = ".."]`) or layout declaration (`{{ extends ".." }}`).
///
/// - `Path`: path relative to `templates` directory
/// - `Root`: path relative to current directory
/// - `Source`: the source string is inlined
pub enum SourceTempl {
    Path(Box<str>),
    Root(Box<str>),
    Source(Box<str>),
}

impl SourceTempl {
    /// Create [`SourceTempl`] from layout declaration.
    ///
    /// Currently, there is no way to define layout as inline, so calling
    /// returned [`SourceTempl::shallow_clone`] will never panic.
    pub fn from_layout(layout: &LayoutTempl) -> syn::Result<SourceTempl> {
        let path = layout.path.value().into_boxed_str();
        match std::fs::exists(path.as_ref()) {
            Ok(true) => {},
            Ok(false) => error!(layout.path, "cannot find file `{path}`"),
            Err(err) => error!(layout.path,"{err}",),
        }
        if layout.root_token.is_some() {
            Ok(SourceTempl::Root(path))
        } else {
            Ok(SourceTempl::Path(path))
        }
    }

    pub fn resolve_source(&self) -> syn::Result<Cow<'_,str>> {
        match self.resolve_path() {
            Some(path) => Ok(error!(!std::fs::read_to_string(path.as_ref())).into()),
            None => match self {
                Self::Source(src) => Ok(Cow::Borrowed(src.as_ref())),
                _ => unreachable!(),
            },
        }
    }

    /// Return `Some` if template is external and have path.
    pub fn resolve_path(&self) -> Option<Box<str>> {
        match self {
            Self::Path(path) => {
                let mut cwd = std::env::current_dir().expect("failed to get current directory");
                cwd.push("templates");
                cwd.push(path.as_ref());
                Some(cwd.to_string_lossy().into_owned().into_boxed_str())
            },
            Self::Root(path) => {
                let mut cwd = std::env::current_dir().expect("failed to get current directory");
                cwd.push(path.as_ref());
                Some(cwd.to_string_lossy().into_owned().into_boxed_str())
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

use crate::syntax::LayoutTempl;

