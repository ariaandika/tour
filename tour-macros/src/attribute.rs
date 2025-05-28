use std::borrow::Cow;
use syn::{punctuated::Punctuated, *};

use crate::shared::{Reload, SourceTempl, error};

// ===== AttrData =====

pub struct AttrData {
    /// `#[path = ".." | root = ".." | source = ".."]`
    pub source: SourceTempl,
    /// `#[path = "debug" | "always" | "never" | <Expr>]`
    pub reload: Reload,
}

impl AttrData {
    pub fn resolve_source(&self) -> Result<Cow<'_,str>> {
        self.source.resolve_source()
    }

    /// Return `Some` if template is external and have path.
    pub fn resolve_path(&self) -> Option<String> {
        self.source.resolve_path()
    }

    /// Parse from derive macro attributes
    pub fn from_attr(attrs: &mut Vec<Attribute>) -> Result<Self> {
        let Some(index) = attrs
            .iter()
            .position(|attr| attr.meta.path().is_ident("template"))
        else {
            error!("`template` attribute missing")
        };

        let attr = attrs.swap_remove(index);

        let Meta::List(input) = attr.meta else {
            error!("expected `#[template(/* .. */)]`")
        };

        let input =
            input.parse_args_with(Punctuated::<MetaNameValue, Token![,]>::parse_terminated)?;

        let mut source = None;
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
                    None => Reload::Expr(input.value),
                });

                if dupl.is_some() {
                    error!("duplicate key `reload`")
                }

                continue;
            }

            let value = str_value(&input.value)?;
            let dupl = source.replace(match &key[..] {
                "path" => SourceTempl::Path(value),
                "root" => SourceTempl::Root(value),
                "source" => SourceTempl::Source(value),
                _ => error!("expected one of `path`, `root`, `source`, or `reload`; found `{key}`"),
            });

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

        Ok(Self { source, reload })
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
    pub fn from_attr(attrs: &mut Vec<Attribute>) -> Result<Self> {
        let Some(index) = attrs
            .iter()
            .position(|attr| attr.meta.path().is_ident("fmt"))
        else {
            return Ok(Self { fmt: None })
        };

        let attr = attrs.swap_remove(index);

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

fn may_str(value: &Expr) -> Option<String> {
    match value {
        Expr::Lit(ExprLit { lit: Lit::Str(lit), .. }) => Some(lit.value()),
        _ => None,
    }
}


