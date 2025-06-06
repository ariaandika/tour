use syn::{punctuated::Punctuated, *};

use crate::shared::{Reload, Source, error};

// ===== AttrData =====

/// Derive macro type level attribute
///
/// Accept input:
///
/// - source: `#[path = ".." | root = ".." | source = ".."]`
/// - block: `#[block = <Ident>]`
/// - reload: `#[path = "debug" | "always" | "never" | <Expr>]`
pub struct AttrData {
    source: Source,
    block: Option<Ident>,
    reload: Reload,
}

impl AttrData {
    /// Parse from derive macro attributes
    pub fn from_attr(attrs: &[Attribute]) -> Result<Self> {
        let mut visitor = Visitor::default();

        for attr in attrs.iter().filter(|e| e.meta.path().is_ident("template")) {
            let attrs = attr.parse_args_with(Punctuated::<MetaNameValue, Token![,]>::parse_terminated)?;

            for MetaNameValue { path, value, .. } in attrs {
                visitor.visit_pair(path.require_ident()?.clone(), value)?;
            }
        }

        let Visitor { source: Some(source), block, reload } = visitor else {
            error!("one of `path`, `root`, or `source` is required")
        };

        Ok(Self { source, block, reload: reload.unwrap_or_default() })
    }

    pub fn source(&self) -> &Source {
        &self.source
    }

    pub fn dir(&self) -> Option<Box<str>> {
        std::path::Path::new(self.source.path()?)
            .parent()
            .map(|e| e.to_string_lossy().into_owned().into_boxed_str())
    }

    pub fn reload(&self) -> &Reload {
        &self.reload
    }

    pub fn block(&self) -> Option<&Ident> {
        self.block.as_ref()
    }
}

// ===== visitor =====

#[derive(Default)]
struct Visitor {
    source: Option<Source>,
    block: Option<Ident>,
    reload: Option<Reload>,
}

impl Visitor {
    fn visit_pair(&mut self, name: Ident, value: Expr) -> Result<()> {
        match () {
            _ if name.eq("path") => self.visit_path(name, value),
            _ if name.eq("root") => self.visit_root(name, value),
            _ if name.eq("source") => self.visit_source(name, value),
            _ if name.eq("block") => self.visit_block(name, value),
            _ if name.eq("reload") => self.visit_reload(name, value),
            _ => error!(name, "no such key"),
        }
    }

    fn visit_path(&mut self, name: Ident, value: Expr) -> Result<()> {
        match self.source.replace(Source::new_path(str_value(&value)?.into_boxed_str(), None)?) {
            Some(_) => error!(name, "only single either of `path`, `root`, or `source` allowed"),
            None => Ok(()),
        }
    }

    fn visit_root(&mut self, name: Ident, value: Expr) -> Result<()> {
        match self.source.replace(Source::new_root(str_value(&value)?.into_boxed_str(), None)?) {
            Some(_) => error!(name, "only single either of `path`, `root`, or `source` allowed"),
            None => Ok(()),
        }
    }

    fn visit_source(&mut self, name: Ident, value: Expr) -> Result<()> {
        match self.source.replace(Source::inline(str_value(&value)?.into_boxed_str())) {
            Some(_) => error!(name, "only single either of `path`, `root`, or `source` allowed"),
            None => Ok(()),
        }
    }

    fn visit_block(&mut self, name: Ident, value: Expr) -> Result<()> {
        match self.block.replace(ident_value(&value)?) {
            Some(_) => error!(name, "duplicate `block` key"),
            None => Ok(()),
        }
    }

    fn visit_reload(&mut self, name: Ident, value: Expr) -> Result<()> {
        let value = match may_str(&value).as_deref() {
            Some("debug") => Reload::Debug,
            Some("always") => Reload::Always,
            Some("never") => Reload::Never,
            Some(s) => error! {
                "expected `debug`, `always`, `never`, or expression, found `{s}`"
            },
            None => Reload::Expr(Box::new(value)),
        };

        match self.reload.replace(value) {
            Some(_) => error!(name, "duplicate `reload` key"),
            None => Ok(()),
        }
    }
}

// ===== Util =====

fn str_value(value: &Expr) -> Result<String> {
    match value {
        Expr::Lit(ExprLit { lit: Lit::Str(lit), .. }) => Ok(lit.value()),
        _ => error!("expected string")
    }
}

fn ident_value(value: &Expr) -> Result<Ident> {
    match value {
        Expr::Path(ExprPath { path, .. }) => path.require_ident().cloned(),
        _ => error!("expected identifier")
    }
}

fn may_str(value: &Expr) -> Option<String> {
    match value {
        Expr::Lit(ExprLit { lit: Lit::Str(lit), .. }) => Some(lit.value()),
        _ => None,
    }
}

