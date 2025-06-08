
// ===== Namespace =====

/// `ToTokens` for public name
pub(crate) struct TemplDisplay;

impl quote::ToTokens for TemplDisplay {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        quote::quote! {::tour::TemplDisplay}.to_tokens(tokens);
    }
}

/// `ToTokens` for public name
pub(crate) struct TemplWrite;

impl quote::ToTokens for TemplWrite {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        quote::quote! {::tour::TemplWrite}.to_tokens(tokens);
    }
}

// ===== Constants =====

pub const DERIVE_ATTRIBUTE: &str = "template";

/// Path resolution.
///
/// user given path:
///
/// - `./layout`, resolve relative from current source file
/// - `layout`, resolve from `templates` directory
/// - `/layout`, resolve from current directory
///
/// currently, rust is unable to get rust source file path,
/// for now, relative path in attribute returns error.
///
/// [issue]: <https://github.com/rust-lang/rust/issuze/54725>
pub(crate) mod path {
    use std::{path::{Path, PathBuf}, rc::Rc};

    use super::error;
    use crate::config::Config;

    pub fn cwd() -> PathBuf {
        std::env::current_dir().expect("current dir")
    }

    pub fn boxed(buf: PathBuf) -> Rc<str> {
        buf.to_string_lossy().into()
    }

    pub fn resolve(mut path: &str, conf: &Config) -> syn::Result<Rc<str>> {
        let mut cwd = cwd();
        match () {
            _ if path.starts_with(".") => error!("cannot get template file using relative path"),
            _ if path.starts_with("/") => path = path.trim_start_matches('/'),
            _ => cwd.push(conf.templ_dir()),
        };
        Ok(resolve_at(path, cwd))
    }

    /// resolve path relative to given directory
    pub fn resolve_at(path: impl AsRef<Path>, cwd: impl Into<PathBuf>) -> Rc<str> {
        let mut cwd = cwd.into();
        cwd.push(path);
        normalize(cwd.as_path())
            .to_string_lossy()
            .into()
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
    (.$s:expr, $($tt:tt)*) => {
        if $s { crate::common::error!($($tt)*) }
    };
    (?$s:expr, $($tt:tt)*) => {
        match $s { Some(ok) => ok, None => crate::common::error!($($tt)*), }
    };
    (!$s:expr, $($tt:tt)*) => {
        match $s { Ok(ok) => ok, Err(err) => crate::common::error!(@proc_macro2::Span::call_site(), $($tt)*, err), }
    };
    (!$s:expr) => {
        match $s { Ok(ok) => ok, Err(err) => crate::common::error!("{err}"), }
    };
    ($s:expr, $($tt:tt)*) => {
        crate::common::error!(@ $s.span(), $($tt)*)
    };
    ($($tt:tt)*) => {
        crate::common::error!(@ proc_macro2::Span::call_site(), $($tt)*)
    };
}

pub(crate) use error;
