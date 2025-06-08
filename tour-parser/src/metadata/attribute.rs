use std::rc::Rc;
use syn::{punctuated::Punctuated, *};

use super::{Metadata, Reload};
use crate::{
    common::{DERIVE_ATTRIBUTE, error, path},
    config::Config,
};

// ===== Visitor =====

pub struct AttrVisitor<'a> {
    conf: &'a Config,
    path: Option<Rc<str>>,
    source: Option<Rc<str>>,
    block: Option<Ident>,
    reload: Option<Reload>,
}

impl<'a> AttrVisitor<'a> {
    /// Derive macro type level attribute
    ///
    /// Accept input:
    ///
    /// - path: `#[path = ".." | source = ".."]`
    /// - block: `#[block = <Ident>]`
    /// - reload: `#[path = "debug" | "always" | "never" | <Expr>]`
    pub fn parse(attrs: &[Attribute], conf: &'a Config) -> Result<Metadata> {
        let mut visitor = Self {
            conf,
            path: None,
            source: None,
            block: None,
            reload: None,
        };

        for attr in attrs.iter().filter(|e| e.meta.path().is_ident(DERIVE_ATTRIBUTE)) {
            let attrs =
                attr.parse_args_with(Punctuated::<MetaNameValue, Token![,]>::parse_terminated)?;

            for MetaNameValue { path, value, .. } in attrs {
                visitor.visit_pair(path.require_ident()?.clone(), value)?;
            }
        }

        let AttrVisitor { path: Some(path), source, block, reload, .. } = visitor else {
            error!("one of `path`, `root`, or `source` is required")
        };

        Ok(Metadata { path, source, reload: reload.unwrap_or_default(), block, })
    }

    fn visit_pair(&mut self, name: Ident, value: Expr) -> Result<()> {
        match () {
            _ if name.eq("path") => self.visit_path(name, value),
            _ if name.eq("source") => self.visit_source(name, value),
            _ if name.eq("block") => self.visit_block(name, value),
            _ if name.eq("reload") => self.visit_reload(name, value),
            _ => error!(name, "no such key"),
        }
    }

    fn visit_path(&mut self, name: Ident, value: Expr) -> Result<()> {
        self.set_path(path::resolve(&str_value(&value)?, self.conf)?, name)
    }

    fn visit_source(&mut self, name: Ident, value: Expr) -> Result<()> {
        self.source = Some(str_value(&value)?.into());
        self.set_path(path::boxed(path::cwd()), name)
    }

    fn set_path(&mut self, source: Rc<str>, span: impl spanned::Spanned) -> Result<()> {
        match self.path.replace(source) {
            Some(_) => error!(span, "only single either of `path`, `root`, or `source` allowed"),
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
            None => Reload::Expr(Rc::new(value)),
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

