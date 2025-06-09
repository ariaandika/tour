use std::{borrow::Cow, fs::read_to_string, rc::Rc};
use syn::*;

use crate::{
    common::{error, path},
    config::Config,
    syntax::LayoutTempl,
};

mod attribute;

use attribute::AttrVisitor;

// ===== Metadata =====

/// Extra information declared outside template file.
#[derive(Debug)]
pub struct Metadata {
    path: Rc<str>,
    source: Option<Rc<str>>,
    reload: Reload,
    block: Option<Ident>,
}

impl Metadata {
    pub(crate) fn new(
        path: Rc<str>,
        source: Option<Rc<str>>,
        reload: Reload,
        block: Option<Ident>,
    ) -> Self {
        Self {
            path,
            source,
            reload,
            block,
        }
    }

    /// Create metadata by parsing [`Attribute`]s.
    pub fn from_attrs(attrs: &[Attribute], conf: &Config) -> Result<Metadata> {
        AttrVisitor::parse(attrs, conf)
    }

    /// Create [`Metadata`] with given path inherited from parent meta.
    pub fn clone_with_path(&self, path: impl AsRef<std::path::Path>) -> Metadata {
        Self {
            path: path::resolve_at(path, self.dir_ref()),
            source: None,
            reload: self.reload.clone(),
            block: None,
        }
    }

    /// Generate layout [`Metadata`] inherited from parent meta.
    pub fn clone_with_layout(&self, layout: &LayoutTempl) -> Metadata {
        Self {
            path: path::resolve_at(layout.path.value(), self.dir_ref()),
            source: None,                // there is no inline layout
            reload: self.reload.clone(), // layout specific reload seems redundant
            block: None,                 // allows select block for a layout ?
        }
    }

    /// Returns inlined source or read source from filesystem.
    pub fn resolve_source(&self) -> Result<Cow<'_, str>> {
        match self.source.as_deref() {
            Some(src) => Ok(src.into()),
            None => Ok(error!(
                !read_to_string(&*self.path),
                "cannot read `{}`: {}", self.path
            )
            .into()),
        }
    }

    pub(crate) fn dir_ref(&self) -> &std::path::Path {
        std::path::Path::new(&*self.path)
            .parent()
            .unwrap_or(std::path::Path::new("/"))
    }

    /// Returns `true` if template is a file, not inlined.
    ///
    /// Internally, its just check if the file exists.
    pub(crate) fn is_file(&self) -> bool {
        std::path::Path::new(&*self.path).is_file()
    }

    pub fn block(&self) -> Option<&Ident> {
        self.block.as_ref()
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn reload(&self) -> &Reload {
        &self.reload
    }
}

// ===== Reload =====

/// Runtime template reload behavior.
#[derive(Clone)]
pub enum Reload {
    Debug,
    Always,
    Never,
    Expr(Rc<Expr>),
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
    /// Returns `Ok(true)` if runtime reload is enabled, otherwise returns `Err(expr)` containing
    /// user defined expression to decide runtime reload.
    pub fn as_bool(&self) -> std::result::Result<bool, &Expr> {
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

