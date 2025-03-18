//! `Template` derive macro
use crate::parser::{LayoutInfo, Reload, SynOutput, SynParser};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::fs;
use syn::{punctuated::Punctuated, *};
use tour_core::parser::{self, Parser, Template};

macro_rules! error {
    (@ $s:expr, $($tt:tt)*) => {
        return Err(Error::new($s, format!($($tt)*)))
    };
    (!$s:expr, $($tt:tt)*) => {
        match $s { Some(ok) => ok, None => error!($($tt)*), }
    };
    (!$s:expr) => {
        match $s { Ok(ok) => ok, Err(err) => error!("{err}"), }
    };
    ($s:expr, $($tt:tt)*) => {
        error!(@ $s.span(), $($tt)*)
    };
    ($($tt:tt)*) => {
        error!(@ Span::call_site(), $($tt)*)
    };
}

/// output code can be split to 4 parts:
///
/// 1. include_source, for file template, an `include_str` to trigger recompile on template change
/// 2. destructor, for named fields, destructor for convenient
/// 3. sources, array of string containing static content
/// 4. statements, the actual rendering or logic code
///
/// input provided via macro attributes
///
pub fn template(input: DeriveInput) -> Result<TokenStream> {
    let DeriveInput { attrs, vis: _, ident, generics, data } = input;

    let (g1,g2,g3) = generics.split_for_impl();

    let attrs = find_template_attr(attrs)?;
    let (source,path) = find_source(&attrs)?;
    let reload = find_reload(&attrs)?;

    // 1. include_source, for file template, an `include_str` to trigger recompile on template change
    let include_source = generate::include_source(&path);

    // 2. destructor, for named fields, destructor for convenient
    let destructor = match data {
        Data::Struct(data) if matches!(data.fields.members().next(),Some(Member::Named(_))) => {
            let fields = data.fields.into_iter().map(|f|f.ident.expect("checked in if guard"));
            quote! { #[allow(unused_variables)] let #ident { #(#fields),* } = self; }
        }
        // unit struct, or unnamed struct does not destructured
        _ => quote! {}
    };

    // the template
    let Template { output: SynOutput { layout, stmts, reload }, statics } = generate::template(&source, reload)?;

    // 3. sources, array of string containing static content
    let sources = generate::sources(&path, &reload, &statics);

    let layout = match layout {
        Some(layout) => template_layout(layout, reload)?,
        None => quote! {{
            self.render_into(writer)
        }},
    };

    Ok(quote! {
        impl #g1 ::tour::Template for #ident #g2 #g3 {
            fn render_into(&self, writer: &mut impl ::tour::Writer) -> ::tour::Result<()> {
                #include_source
                #destructor
                #(#sources)*
                #(#stmts)*
                Ok(())
            }

            fn render_layout_into(&self, writer: &mut impl ::tour::Writer) -> ::tour::Result<()>
                #layout
        }

        impl #g1 ::tour::Display for #ident #g2 #g3 {
            fn display(&self, f: &mut impl ::tour::Writer) -> ::tour::Result<()> {
                self.render_into(f)
            }
        }
    })
}

fn template_layout(templ: LayoutInfo, reload: Reload) -> Result<TokenStream> {
    let LayoutInfo { source, is_root } = templ;
    let (source,path) = fs_read(source, !is_root)?;

    // 1. include_source, for file template, an `include_str` to trigger recompile on template change
    let include_source = generate::include_source(&path);

    // 2. destructor, no destructor in layout

    // the template
    let Template { output: SynOutput { layout, stmts, reload }, statics } = generate::template(&source, reload)?;

    // 3. sources, array of string containing static content
    let sources = generate::sources(&path, &reload, &statics);

    if layout.is_some() {
        error!("TODO: layout in layout is not yet supported")
    }

    // layout specific `yield` interpretation
    let layout_inner = quote! {
        let layout_inner = &*self;
    };

    Ok(quote! {{
        #include_source
        #layout_inner
        #(#sources)*
        #(#stmts)*
        Ok(())
    }})
}

fn find_template_attr(mut attrs: Vec<Attribute>) -> Result<Vec<MetaNameValue>> {
    let index = attrs.iter().position(|attr|attr.meta.path().is_ident("template"));
    let Some(attr) = index.map(|e|attrs.swap_remove(e)) else {
        error!("`template` attribute missing")
    };
    let Meta::List(meta_list) = attr.meta else {
        error!("expected `#[template(/* .. */)]`")
    };

    Ok(meta_list
        .parse_args_with(Punctuated::<MetaNameValue,Token![,]>::parse_terminated)?
        .into_iter()
        .collect::<Vec<_>>())
}

/// return (source,path)
fn find_source(attrs: &Vec<MetaNameValue>) -> Result<(String,Option<String>)> {
    fn str_value(value: &Expr) -> Result<String> {
        match value {
            Expr::Lit(ExprLit { lit: Lit::Str(lit), .. }) => Ok(lit.value()),
            _ => error!("expected string")
        }
    }

    for MetaNameValue { path, value, .. } in attrs {
        match () {
            _ if path.is_ident("path") => return fs_read(str_value(value)?, true),
            _ if path.is_ident("root") => return fs_read(str_value(value)?, false),
            _ if path.is_ident("source") => return Ok((str_value(value)?,None)),
            _ => continue,
        }
    }

    error!("require `path`, `root` or `source`")
}

fn fs_read(path: String, is_template: bool) -> Result<(String, Option<String>)> {
    let mut abs_path = error!(!std::env::current_dir());
    if is_template {
        abs_path.push("templates")
    }
    abs_path.push(&path);
    let path = error!(!abs_path.to_str(),"non utf8 path").to_owned();
    match fs::read_to_string(&path) {
        Ok(ok) => Ok((ok,Some(path))),
        Err(err) => error!("failed to read template `{path}`: {err}"),
    }
}

fn find_reload(attrs: &Vec<MetaNameValue>) -> Result<Reload> {
    for MetaNameValue { path, value, .. } in attrs {
        if !path.is_ident("reload") {
            continue;
        }

        // reload = debug
        // reload = always
        // reload = never
        // reload = "not(cfg(test))"

        match value {
            Expr::Path(ExprPath { path, .. }) if path.is_ident("debug") => return Ok(Reload::Debug),
            Expr::Path(ExprPath { path, .. }) if path.is_ident("always") => return Ok(Reload::Always),
            Expr::Path(ExprPath { path, .. }) if path.is_ident("never") => return Ok(Reload::Never),
            Expr::Lit(ExprLit { lit: Lit::Str(lit), .. }) =>
                return syn::parse_str(&lit.value()).map(Reload::Expr),
            _ => continue,
        }
    }

    Ok(if cfg!(feature = "dev-reload") { Reload::Debug } else { Reload::Never })
}

mod generate {
    use super::*;

    pub fn include_source(path: &Option<String>) -> Option<TokenStream> {
        path.as_ref().map(|path|quote!{const _: &str = include_str!(#path);})
    }

    pub fn template(source: &str, reload: Reload) -> Result<Template<'_, SynOutput>> {
        match Parser::new(source, SynParser::new(reload)).parse() {
            Ok(ok) => Ok(ok),
            Err(parser::Error::Generic(err)) => error!("{err}"),
        }
    }

    pub fn sources(path: &Option<String>, reload: &Reload, statics: &[&str]) -> [TokenStream;2] {
        match (path.is_some(), reload.as_bool()) {
            (true,Ok(true)) => [
                quote!{ let sources = ::std::fs::read_to_string(#path)?; },
                quote!{ let sources = ::tour::Parser::new(&sources,::tour::NoopParser).parse()?.statics; },
            ],
            (true,Ok(false)) | (false,Ok(false)) => <_>::default(),
            (true, Err(cond)) => [
                quote! {
                    let sources = if #cond {
                        ::std::fs::read_to_string(#path)?
                    } else {
                        String::new()
                    };
                },
                quote! {
                    #[allow(unused_variables)]
                    let sources = if #cond {
                        ::tour::Parser::new(&sources,::tour::NoopParser).parse()?.statics
                    } else {
                        []
                    };
                }
            ],
            (false, _) => [quote!{ let sources = [#(#statics),*]; },<_>::default()],
        }
    }
}

