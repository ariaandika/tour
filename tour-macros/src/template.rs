use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::fs;
use syn::{punctuated::Punctuated, spanned::Spanned, *};
use tour_parser::{parser::{self, Parser}, token::LayoutTempl, Template};

macro_rules! error {
    (@ $s:expr, $($tt:tt)*) => {
        return Err(Error::new($s, format!($($tt)*)))
    };
    (!$s:expr, $($tt:tt)*) => {
        match $s {
            Some(ok) => ok,
            None => error!($($tt)*),
        }
    };
    (!$s:expr) => {
        match $s {
            Ok(ok) => ok,
            Err(err) => error!("{err}"),
        }
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
/// - include_source, for file template, an `include_str` to trigger recompile on template change
/// - destructor, for named fields, destructor for convenient
/// - sources, array of string containing static content
/// - statements, the actual rendering or logic code
///
/// input provided via macro attributes
///
pub fn template(input: DeriveInput) -> Result<TokenStream> {
    match &input.data {
        Data::Struct(_) => template_struct(input),
        Data::Enum(_) => error!(input, "enum not yet supported"),
        Data::Union(_) => error!(input, "union not supported"),
    }
}

fn template_struct(input: DeriveInput) -> Result<TokenStream> {
    let DeriveInput { attrs, vis: _, ident, generics, data } = input;
    let Data::Struct(data) = data else { unreachable!() };

    let (g1, g2, g3) = generics.split_for_impl();

    let attrs = find_template_attr(attrs)?;
    let (source,path) = find_source(&attrs)?;

    // include_source, for file template, an `include_str` to trigger recompile on template change
    let include_source = path.as_ref().map(|path|quote!{const _: &str = include_str!(#path);});

    // destructor, for named fields, destructor for convenient
    let destructor = match () {
        _ if matches!(data.fields.members().next(),Some(Member::Named(_))) => {
            let fields = data.fields.into_iter().map(|f|f.ident.expect("checked in if guard"));
            quote! { let #ident { #(#fields),* } = self; }
        }
            // unit struct, or unnamed struct does not destructured
        _ => quote! { }
    };

    // the template
    let Template { layout, stmts, statics } = match Parser::new(&source).parse() {
        Ok(ok) => ok,
        Err(err) => return Err(match err {
            parser::Error::Generic(err) => Error::new(Span::call_site(), err),
            parser::Error::Syn(error) => error,
        })
    };

    // sources, array of string containing static content
    let sources = {
        match (path.is_some(), cfg!(debug_assertions)) {
            (true,true) => quote!{ let sources = ::tour::Parser::new(&::std::fs::read_to_string(#path)?).parse()?.statics; },
            (false, true) => quote!{ let sources = [#(#statics),*]; },
            (true,false) | (false,false) => quote! { }
        }
    };

    let layout = match layout {
        Some(layout) => template_layout(layout)?,
        None => quote! {{
            self.render_into(writer)
        }},
    };

    Ok(quote! {
        impl #g1 ::tour::Template for #ident #g2 #g3 {
            fn render_into(&self, writer: &mut impl ::tour::Writer) -> ::tour::template::Result<()> {
                #include_source
                #destructor
                #sources
                #(#stmts)*
                Ok(())
            }

            fn render_layout_into(&self, writer: &mut impl ::tour::Writer) -> ::tour::template::Result<()>
                #layout
        }

        impl #g1 ::tour::Display for #ident #g2 #g3 {
            fn display(&self, f: &mut impl ::tour::Writer) -> ::tour::template::Result<()> {
                self.render_into(f)
            }
        }
    })
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

fn template_layout(templ: LayoutTempl) -> Result<TokenStream> {
    let LayoutTempl { layout_token: _, root_token, source } = templ;
    let (source,path) = fs_read(source.value(), root_token.is_none())?;

    // include_source, for file template, an `include_str` to trigger recompile on template change
    let include_source = path.as_ref().map(|path|quote!{const _: &str = include_str!(#path);});

    // the template
    let Template { layout, stmts, statics } = match Parser::new(&source).parse() {
        Ok(ok) => ok,
        Err(err) => return Err(match err {
            parser::Error::Generic(err) => Error::new(Span::call_site(), err),
            parser::Error::Syn(error) => error,
        })
    };

    // sources, array of string containing static content
    let sources = {
        match (path.is_some(), cfg!(debug_assertions)) {
            (true,true) => quote!{ let sources = ::tour::Parser::new(&::std::fs::read_to_string(#path)?).parse()?.statics; },
            (false, true) => quote!{ let sources = [#(#statics),*]; },
            (true,false) | (false,false) => quote! { }
        }
    };

    if layout.is_some() {
        error!("layout in layout is not yet supported")
    }

    // layout specific `yield` interpretation
    let layout_inner = quote! {
        let layout_inner = &*self;
    };

    Ok(quote! {{
        #include_source
        #layout_inner
        #sources
        #(#stmts)*
        Ok(())
    }})
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

