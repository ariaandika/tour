use syn::{punctuated::Punctuated, spanned::Spanned, *};

use crate::{
    data::Metadata,
    shared::{Reload, SourceTempl, error},
};

// ===== AttrData =====

/// Derive macro type level attribute
///
/// Accept input:
///
/// - source: `#[path = ".." | root = ".." | source = ".."]`
/// - block: `#[block = <Ident>]`
/// - reload: `#[path = "debug" | "always" | "never" | <Expr>]`
pub struct AttrData {
    source: SourceTempl,
    block: Option<Ident>,
    reload: Reload,
}

impl AttrData {
    /// Parse from derive macro attributes
    pub fn from_attr(attrs: &[Attribute]) -> Result<Self> {
        let Some(index) = attrs
            .iter()
            .position(|attr| attr.meta.path().is_ident("template"))
        else {
            error!("`template` attribute missing")
        };

        let attr = attrs[index].clone();

        let Meta::List(input) = attr.meta else {
            error!("expected `#[template(/* .. */)]`")
        };

        let input =
            input.parse_args_with(Punctuated::<MetaNameValue, Token![,]>::parse_terminated)?;

        let mut source = None;
        let mut block = None;
        let mut reload = None;

        for input in input {
            let key = input.path.require_ident()?.to_string();

            if key == "reload" {
                let dupl = reload.replace(match may_str(&input.value).as_deref() {
                    Some("debug") => Reload::Debug,
                    Some("always") => Reload::Always,
                    Some("never") => Reload::Never,
                    Some(s) => {
                        error!("expected `debug`, `always`, `never`, or expression, found `{s}`")
                    }
                    None => Reload::Expr(Box::new(input.value)),
                });

                if dupl.is_some() {
                    error!("duplicate key `reload`")
                }

                continue;
            }

            if key == "block" {
                let name = ident_value(&input.value)?;
                let dupl = block.replace(name);

                if dupl.is_some() {
                    error!("duplicate key `block`")
                }

                continue;
            }

            let path = str_value(&input.value)?.into_boxed_str();
            let me = match &key[..] {
                "path" => SourceTempl::Path(path),
                "root" => SourceTempl::Root(path),
                "source" => SourceTempl::Source(path),
                _ => error!("expected one of `path`, `root`, `source`, or `reload`; found `{key}`"),
            };

            if let Some(path) = me.resolve_path() {
                match std::fs::exists(path.as_ref()) {
                    Ok(true) => (),
                    Ok(false) => error!(input.value, "cannot find file `{path}`"),
                    Err(err) => error!(input.value, "{err}",),
                }
            }

            let dupl = source.replace(me);

            if dupl.is_some() {
                error!("duplicate key `path`, `root`, or `source`")
            }
        }

        let Some(source) = source else {
            error!("expected one of `path`, `root`, `source`, or `reload`")
        };

        let reload = reload.unwrap_or(if cfg!(feature = "dev-reload") {
            Reload::Debug
        } else {
            Reload::Never
        });

        Ok(Self { source, block, reload })
    }

    pub fn source(&self) -> &SourceTempl {
        &self.source
    }

    pub fn reload(&self) -> &Reload {
        &self.reload
    }

    pub fn to_meta(&self) -> Metadata {
        let path = self.source.resolve_path();
        Metadata::new(path, self.reload.clone(), self.block.clone())
    }
}

impl Reload {
    pub fn as_bool(&self) -> std::result::Result<bool,&Expr> {
        match self {
            Reload::Debug => Ok(cfg!(debug_assertions)),
            Reload::Always => Ok(true),
            Reload::Never => Ok(false),
            Reload::Expr(expr) => Err(expr),
        }
    }
}

// ===== AttrField =====

pub struct AttrField {
    /// `#[fmt(display | debug)]`
    pub fmt: Option<FmtTempl>,
}

impl AttrField {
    pub fn from_attr(attrs: &[Attribute]) -> Result<Self> {
        let Some(index) = attrs
            .iter()
            .position(|attr| attr.meta.path().is_ident("fmt"))
        else {
            return Ok(Self { fmt: None })
        };

        let attr = attrs[index].clone();

        let Meta::List(input) = attr.meta else {
            error!("expected `#[fmt(/* .. */)]`")
        };

        let input: Ident = input.parse_args()?;

        let fmt = match input.to_string().as_str() {
            "display" => FmtTempl::Display,
            "debug" => FmtTempl::Debug,
            val => error!("expected one of `display` or `debug`; found `{val}`"),
        };

        Ok(Self { fmt: Some(fmt) })
    }
}

pub enum FmtTempl {
    Display,
    Debug,
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


