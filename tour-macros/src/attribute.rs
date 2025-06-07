use syn::{punctuated::Punctuated, *};

use crate::{
    config::Config,
    data::Metadata,
    shared::{Reload, error, path},
};

/// Derive macro type level attribute
///
/// Accept input:
///
/// - path: `#[path = ".." | source = ".."]`
/// - block: `#[block = <Ident>]`
/// - reload: `#[path = "debug" | "always" | "never" | <Expr>]`
pub fn generate_meta(attrs: &[Attribute], conf: &Config) -> Result<Metadata> {
    let mut visitor = Visitor::new(conf);

    for attr in attrs.iter().filter(|e| e.meta.path().is_ident("template")) {
        let attrs = attr.parse_args_with(Punctuated::<MetaNameValue, Token![,]>::parse_terminated)?;

        for MetaNameValue { path, value, .. } in attrs {
            visitor.visit_pair(path.require_ident()?.clone(), value)?;
        }
    }

    let Visitor { path: Some(path), source, block, reload, .. } = visitor else {
        error!("one of `path`, `root`, or `source` is required")
    };

    Ok(Metadata::new(path, source, reload.unwrap_or_default(), block))
}

// ===== Visitor =====

struct Visitor<'a> {
    conf: &'a Config,
    path: Option<Box<str>>,
    source: Option<Box<str>>,
    block: Option<Ident>,
    reload: Option<Reload>,
}

impl<'a> Visitor<'a> {
    fn new(
        conf: &'a Config,
    ) -> Self {
        Self {
            conf,
            path: None,
            source: None,
            block: None,
            reload: None,
        }
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
        self.source = Some(str_value(&value)?.into_boxed_str());
        self.set_path(path::boxed(path::cwd()), name)
    }

    fn set_path(&mut self, source: Box<str>, span: impl spanned::Spanned) -> Result<()> {
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

