//! `Template` derive macro
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::*;
use tour_core::{ParseError, Parser};

use crate::{
    attribute::{AttrData, AttrField, FmtTempl},
    parser::SynParser,
    shared::{error, Reload, SourceTempl, TemplDisplay, TemplWrite},
};

/// output code can be split to 4 parts:
///
/// 1. include_source, for file template, an `include_str` to trigger recompile on template change
/// 2. destructor, for named fields, destructor for convenient
/// 3. sources, array of string containing static content, omited on release
/// 4. statements, the actual rendering or logic code
///
/// input provided via macro attributes
///
pub fn template(input: DeriveInput) -> Result<TokenStream> {
    let DeriveInput { mut attrs, vis: _, ident, generics, mut data } = input;

    let (g1,g2,g3) = generics.split_for_impl();

    let attr = AttrData::from_attr(&mut attrs)?;
    let path = attr.resolve_path();

    // 1. include_source, for file template, an `include_str` to trigger recompile on template change
    let include_source = generate::include_str_source(path.as_deref());

    // 2. destructor, for named fields, destructor for convenient
    let destructor = match &data {
        Data::Struct(data) if matches!(data.fields.members().next(),Some(Member::Named(_))) => {
            let fields = data.fields.iter().map(|f|f.ident.as_ref().expect("checked in if guard"));
            quote! { let #ident { #(#fields),* } = self; }
        }
        // unit struct, or unnamed struct does not destructured
        _ => quote! {}
    };

    // field with `#[fmt(display)]` attribute
    let displays = match &mut data {
        Data::Struct(data) if matches!(data.fields.members().next(),Some(Member::Named(_))) => {
            let mut displays = quote! {};

            for field in &mut data.fields {
                let attr = AttrField::from_attr(&mut field.attrs)?;
                let id = field.ident.as_ref().cloned().unwrap();

                match &attr.fmt {
                    Some(FmtTempl::Display) => displays.extend(quote! {
                        let #id = ::tour::Display(&#id);
                    }),
                    Some(FmtTempl::Debug) => displays.extend(quote! {
                        let #id = ::tour::Display(&#id);
                    }),
                    None => continue,
                }
            }

            displays
        }
        // unit struct, or unnamed struct cannot have display attribute
        _ => quote! {}
    };

    // the template
    let SynParser { layout_source, root: stmts, reload, statics, .. } = generate::template(attr.resolve_source()?.as_ref(), attr.reload.clone())?;

    // 3. sources, array of string containing static content, omited on release
    let sources = generate::sources(path.as_deref(), &reload, &statics);

    let layout = match layout_source {
        Some(layout) => {
            let layout = template_layout(layout, reload)?;
            Some(quote! {
                fn render_layout_into(&self, writer: &mut impl #TemplWrite)
                    -> ::tour::Result<()> #layout
            })
        },
        None => None,
    };

    Ok(quote! {
        #[automatically_derived]
        impl #g1 ::tour::Template for #ident #g2 #g3 {
            fn render_into(&self, writer: &mut impl #TemplWrite) -> ::tour::Result<()> {
                #include_source
                #destructor
                #displays
                #(#sources)*
                #(#stmts)*
                Ok(())
            }

            #layout
        }

        #[automatically_derived]
        impl #g1 #TemplDisplay for #ident #g2 #g3 {
            fn display(&self, f: &mut impl #TemplWrite) -> ::tour::Result<()> {
                self.render_into(f)
            }
        }
    })
}

fn template_layout(first_templ_source: SourceTempl, reload: Reload) -> Result<TokenStream> {
    let path = first_templ_source.resolve_path();
    let source = first_templ_source.resolve_source()?;

    // 1. include_source, for file template, an `include_str` to trigger recompile on template change
    let include_source = generate::include_str_source(path.as_deref());

    // 2. destructor, no destructor in layout

    // no `#[fmt(display)]` in layout

    // the template
    let SynParser { layout_source: layout, root: stmts, reload, statics, .. } = generate::template(&source, reload)?;

    // 3. sources, array of string containing static content
    let sources = generate::sources(path.as_deref(), &reload, &statics);

    if layout.is_none() {
        return Ok(quote! {{
            #include_source
            let layout_inner = &*self;
            #(#sources)*
            #(#stmts)*
            Ok(())
        }});
    }

    // Nested Layout

    let mut reload = reload;
    let mut layout = layout;
    let mut counter = 0;

    let mut name_inner = format_ident!("InnerLayout{counter}");
    let mut inner = quote! {
        struct #name_inner<S>(S);

        impl<S> #TemplDisplay for #name_inner<S> where S: #TemplDisplay {
            fn display(&self, writer: &mut impl #TemplWrite) -> ::tour::Result<()> {
                #include_source
                let layout_inner = &self.0;
                #(#sources)*
                #(#stmts)*
                Ok(())
            }
        }
    };

    while let Some(layout_source) = layout.take() {
        counter += 1;
        let path = layout_source.resolve_path();
        let source = layout_source.resolve_source()?;

        let include_source = generate::include_str_source(path.as_deref());
        let SynParser { layout_source: l1, root: stmts, reload: r1, statics, .. } = generate::template(&source, reload)?;
        let sources = generate::sources(path.as_deref(), &r1, &statics);

        let name = format_ident!("InnerLayout{counter}");
        inner = quote! {
            struct #name<S>(S);

            impl<S> #TemplDisplay for #name<S> where S: #TemplDisplay {
                fn display(&self, writer: &mut impl #TemplWrite) -> ::tour::Result<()> {
                    #inner

                    #include_source
                    let layout_inner = #name_inner(&self.0);
                    #(#sources)*
                    #(#stmts)*
                    Ok(())
                }
            }
        };

        name_inner = name;
        layout = l1;
        reload = r1;
    }

    Ok(quote! {{
        #inner
        #TemplDisplay::display(&#name_inner(self), writer)
    }})
}

mod generate {
    use super::*;

    /// Generate `include_str!("")`
    pub fn include_str_source(path: Option<&str>) -> Option<TokenStream> {
        path.map(|path|quote!{const _: &str = include_str!(#path);})
    }

    pub fn template(source: &str, reload: Reload) -> Result<SynParser> {
        match Parser::new(source, SynParser::new(reload)).parse() {
            Ok(ok) => Ok(ok),
            Err(ParseError::Generic(err)) => error!("{err}"),
        }
    }

    pub fn sources(path: Option<&str>, reload: &Reload, statics: &[String]) -> [TokenStream;2] {
        match (path.is_some(), reload.as_bool()) {
            (true,Ok(true)) => [
                quote!{ let sources = ::std::fs::read_to_string(#path)?; },
                quote!{ let sources = ::tour::Parser::new(&sources,::tour::StaticVisitor::new()).parse()?.statics; },
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
                    let sources = if #cond {
                        ::tour::Parser::new(&sources,::tour::StaticVisitor::new()).parse()?.statics
                    } else {
                        []
                    };
                }
            ],
            (false, _) => [quote!{ let sources = [#(#statics),*]; },<_>::default()],
        }
    }
}

