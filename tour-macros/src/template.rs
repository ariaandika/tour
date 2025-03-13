use std::fs;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, spanned::Spanned, *};

macro_rules! error {
    (@ $s:expr, $($tt:tt)*) => {
        return Err(Error::new($s, format!($($tt)*)))
    };
    ($s:expr, $($tt:tt)*) => {
        error!(@ $s.span(), $($tt)*)
    };
    ($($tt:tt)*) => {
        error!(@ proc_macro2::Span::call_site(), $($tt)*)
    };
}

pub fn template(input: DeriveInput) -> Result<TokenStream> {
    match &input.data {
        Data::Struct(data) if data.fields.iter().next().map(|e|e.ident.is_some()).unwrap_or(false) => {
            template_struct(input)
        },
        Data::Struct(_) => error!(input, "named struct only"),
        Data::Enum(_) => error!(input, "enum not yet supported"),
        Data::Union(_) => error!(input, "union not supported"),
    }
}

fn template_struct(input: DeriveInput) -> Result<TokenStream> {
    let DeriveInput { mut attrs, vis: _, ident, generics, data } = input;
    let Data::Struct(data) = data else { unreachable!() };

    let (g1, g2, g3) = generics.split_for_impl();
    let fields = data.fields.into_iter().map(|f|f.ident.unwrap());

    let cwd = match std::env::current_dir() {
        Ok(ok) => ok,
        Err(err) => error!("{err}")
    };

    let (tour_parser::Template { extends: _, stmts, statics },path) = {
        let index = attrs.iter().position(|attr|attr.meta.path().is_ident("template"));
        let Some(attr) = index.map(|e|attrs.swap_remove(e)) else {
            error!("`template` attribute missing")
        };
        let Meta::List(meta_list) = attr.meta else {
            error!("expected `#[template(/* .. */)]`")
        };

        let args = meta_list
            .parse_args_with(Punctuated::<MetaNameValue,Token![,]>::parse_terminated)?
            .into_iter()
            .collect::<Vec<_>>();

        let (source,path) = find_source(&args)?;
        let result = tour_parser::parser::Parser::new(&source).parse();
        let template = match result {
            Ok(ok) => ok,
            Err(err) => return Err(match err {
                tour_parser::parser::Error::Generic(err) => Error::new(proc_macro2::Span::call_site(), err),
                tour_parser::parser::Error::Syn(error) => error,
            })
        };

        (template,path)
    };

    let include_source = match &path {
        Some(path) => {
            let cwd = cwd.join(path);
            let cwd = cwd.to_str().unwrap_or("<NON-UTF8-PATH>");
            Some(quote! {const _: &str = include_str!(#cwd);})
        },
        None => None,
    };

    let sources = {
        let is_dynamic = path.is_some() && cfg!(debug_assertions);

        match is_dynamic {
            true => {
                let path = cwd.join(path.unwrap()).to_string_lossy().to_string();
                quote! {
                    let sources = ::tour::Parser::new(&::std::fs::read_to_string(#path)?).parse()?.statics;
                }
            }
            false => {
                let statics: Expr = syn::parse_quote!([#(#statics),*]);
                quote! {
                    let sources = #statics;
                }
            }
        }
    };

    Ok(quote! {
        impl #g1 ::tour::Template for #ident #g2 #g3 {
            fn render_into(&self, writer: &mut impl ::tour::Renderer) -> ::tour::template::Result<()> {
                #include_source
                let #ident { #(#fields)* } = self;
                #sources
                #(#stmts)*
                Ok(())
            }
        }
    })
}

/// return (source,path)
fn find_source(args: &Vec<MetaNameValue>) -> Result<(String,Option<String>)> {
    fn str_value(value: &Expr) -> Result<String> {
        match value {
            Expr::Lit(ExprLit { lit: Lit::Str(lit), .. }) => Ok(lit.value()),
            _ => error!("expected string")
        }
    }

    for MetaNameValue { path, value, .. } in args {
        match () {
            _ if path.is_ident("path") => {
                let path = format!("templates/{}",str_value(value)?);
                return match fs::read_to_string(&path) {
                    Ok(ok) => Ok((ok,Some(path))),
                    Err(err) => error!("failed to read template `{path}`: {err}"),
                }
            },
            _ if path.is_ident("root") => {
                let path = str_value(value)?;
                return match fs::read_to_string(&path) {
                    Ok(ok) => Ok((ok,Some(path))),
                    Err(err) => error!("failed to read template `{path}`: {err}"),
                }
            },
            _ if path.is_ident("source") => return Ok((str_value(value)?,None)),
            _ => continue,
        }
    }

    error!("required `path`, `root` or `source`")
}

