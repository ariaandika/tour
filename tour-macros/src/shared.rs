
// ===== Path =====

pub mod path {
    //! Path resolution.
    //!
    //! user given path:
    //!
    //! - `./layout`, resolve relative from current source file
    //! - `layout`, resolve from `templates` directory
    //! - `/layout`, resolve from current directory
    //!
    //! currently, rust is unable to get rust source file path,
    //! for now, relative path in attribute returns error.
    //!
    //! [issue]: <https://github.com/rust-lang/rust/issuze/54725>
    use std::path::{Path, PathBuf};

    use crate::{config::Config, shared::error};

    pub fn cwd() -> PathBuf {
        std::env::current_dir().expect("current dir")
    }

    pub fn boxed(buf: PathBuf) -> Box<str> {
        buf.to_string_lossy().into_owned().into_boxed_str()
    }

    pub fn resolve(mut path: &str, conf: &Config) -> syn::Result<Box<str>> {
        let mut cwd = cwd();
        match () {
            _ if path.starts_with(".") => error!("cannot get template file using relative path"),
            _ if path.starts_with("/") => path = path.trim_start_matches('/'),
            _ => cwd.push(conf.templ_dir()),
        };
        Ok(resolve_at(path, cwd))
    }

    /// resolve path relative to given directory
    pub fn resolve_at(path: impl AsRef<Path>, cwd: impl Into<PathBuf>) -> Box<str> {
        let mut cwd = cwd.into();
        cwd.push(path);
        normalize(cwd.as_path())
            .to_string_lossy()
            .into_owned()
            .into_boxed_str()
    }

    /// Copied from [cargo][1]
    ///
    /// [1]: https://github.com/rust-lang/cargo/blob/fede83ccf973457de319ba6fa0e36ead454d2e20/src/cargo/util/paths.rs#L61
    pub fn normalize(path: &Path) -> PathBuf {
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
}

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

impl std::fmt::Debug for Reload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Debug => write!(f, "Reload::Debug"),
            Self::Always => write!(f, "Reload::Always"),
            Self::Never => write!(f, "Reload::Never"),
            Self::Expr(_) => write!(f, "Reload::<Expr>"),
        }
    }
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
