use syn::Result;
use std::path::{Path, PathBuf};

use crate::syntax::LayoutTempl;

// ===== Namespace =====

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

// ===== Reload =====

/// Runtime template reload behavior.
#[derive(Clone)]
pub enum Reload {
    Debug,
    Always,
    Never,
    Expr(Box<syn::Expr>),
}

impl Default for Reload {
    fn default() -> Self {
        if cfg!(feature = "dev-reload") {
            Reload::Debug
        } else {
            Reload::Never
        }
    }
}

impl Reload {
    pub fn as_bool(&self) -> std::result::Result<bool,&syn::Expr> {
        match self {
            Reload::Debug => Ok(cfg!(debug_assertions)),
            Reload::Always => Ok(true),
            Reload::Never => Ok(false),
            Reload::Expr(expr) => Err(expr),
        }
    }
}

// ===== Source =====

/// Template reference.
#[derive(Debug)]
pub struct Source {
    path: Option<Box<str>>,
    source: Option<Box<str>>,
}

impl Source {
    /// Create [`Source`] from layout declaration.
    pub fn from_layout(layout: &LayoutTempl, cwd: Option<Box<str>>) -> Result<Source> {
        let path = layout.path.value().into_boxed_str();
        if
            Path::new(path.as_ref()).is_relative() ||
            layout.root_token.is_some()
        {
            Self::new_root(path, cwd)
        } else {
            Self::new_path(path, cwd)
        }
    }

    pub fn new_path(path: Box<str>, cwd: Option<Box<str>>) -> Result<Self> {
        let mut cwd = match cwd {
            None => std::env::current_dir().expect("failed to get current directory"),
            Some(cwd) => cwd.as_ref().into(),
        };
        cwd.push("templates");
        cwd.push(path.as_ref());
        let path = normalize_path(cwd.as_path())
            .to_string_lossy()
            .into_owned()
            .into_boxed_str();
        Ok(Self {
            path: Some(path),
            source: None,
        })
    }

    pub fn new_root(root: Box<str>, cwd: Option<Box<str>>) -> Result<Self> {
        let mut cwd = match cwd {
            None => std::env::current_dir().expect("failed to get current directory"),
            Some(cwd) => cwd.as_ref().into(),
        };
        cwd.push(root.as_ref());
        let path = normalize_path(cwd.as_path())
            .to_string_lossy()
            .into_owned()
            .into_boxed_str();
        Ok(Self { path: Some(path), source: None })
    }

    pub fn inline(source: Box<str>) -> Self {
        Self { path: None, source: Some(source) }
    }

    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }

    pub fn clone_path(&self) -> Option<Box<str>> {
        self.path.as_ref().cloned()
    }

    pub fn resolve_source(&self) -> Result<Box<str>> {
        if self.source.is_some() {
            return Ok(self.source.as_deref().expect("is_some").into());
        }

        let path = self.path.as_deref().expect("constructor is either path or source");
        match std::fs::read_to_string(path) {
            Ok(ok) => Ok(ok.into_boxed_str()),
            Err(err) => error!("cannot read `{path}`: {err}"),
        }
    }
}

// ===== utils =====

/// Copied from [cargo][1]
///
/// [1]: https://github.com/rust-lang/cargo/blob/fede83ccf973457de319ba6fa0e36ead454d2e20/src/cargo/util/paths.rs#L61
pub fn normalize_path(path: &Path) -> PathBuf {
    use std::path::Component;
    let mut components = path.components().peekable();
    let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
        components.next();
        PathBuf::from(c.as_os_str())
    } else {
        PathBuf::new()
    };

    for component in components {
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                ret.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                ret.pop();
            }
            Component::Normal(c) => {
                ret.push(c);
            }
        }
    }
    ret
}

// ===== macros =====

/// Everything will return `Result<T, syn::Error>`
///
/// `error!(?option, "{}", error)`, unwrap option with error as standard `format!`.
///
/// `error!(!result, "{}", error)`, unwrap result with context.
///
/// `error!(!result)`, unwrap result.
///
/// `error!(attr, "`{path}`: {}")`, standard `format!` with `attr`s span.
///
/// `error!("{}",error)`, standard `format!`
macro_rules! error {
    (@ $s:expr, $($tt:tt)*) => {
        return Err(syn::Error::new($s, format!($($tt)*)))
    };
    (dbg $($tt:tt)*) => {{
        let me = $($tt)*;
        panic!("{:?}", me);
        me
    }};
    (?$s:expr, $($tt:tt)*) => {
        match $s { Some(ok) => ok, None => crate::shared::error!($($tt)*), }
    };
    (!$s:expr, $($tt:tt)*) => {
        match $s { Ok(ok) => ok, Err(err) => crate::shared::error!(@proc_macro2::Span::call_site(), $($tt)*, err), }
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
