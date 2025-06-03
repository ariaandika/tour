//! `Template` derive macro
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::*;
use tour_core::{ParseError, Parser};

use crate::{
    attribute::{AttrData, AttrField, FmtTempl},
    codegen,
    shared::{Reload, SourceTempl, TemplDisplay, TemplWrite, error},
    visitor::{SynVisitor, Template},
};

pub fn template(input: DeriveInput) -> Result<TokenStream> {
    let DeriveInput { mut attrs, vis: _, ident, generics, mut data } = input;

    let attr = AttrData::from_attr(&mut attrs)?;
    let path = attr.resolve_path();


    // destructor, for convenient, named fields only
    let destructor = match &data {
        Data::Struct(data) if matches!(data.fields.members().next(),Some(Member::Named(_))) => {
            let fields = data.fields.iter().map(|f|f.ident.as_ref().expect("checked in if guard"));
            quote! { let #ident { #(#fields),* } = self; }
        }
        // named fields only
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
        // named fields only
        _ => quote! {}
    };

    // codegen

    let include_source = generate::include_str_source(path.as_deref());
    let (Template { layout, statics, .. },body) = generate::template(attr.resolve_source()?.as_ref(), &attr)?;
    let sources = generate::sources(path.as_deref(), &attr.reload, &statics);

    let layout = match layout {
        Some(layout) => {
            let layout = template_layout(layout, attr)?;
            Some(quote! {
                fn render_layout_into(&self, writer: &mut impl #TemplWrite)
                    -> ::tour::Result<()> #layout
            })
        },
        None => None,
    };

    let (g1,g2,g3) = generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #g1 ::tour::Template for #ident #g2 #g3 {
            fn render_into(&self, writer: &mut impl #TemplWrite) -> ::tour::Result<()> {
                #include_source
                #destructor
                #displays
                #(#sources)*
                #body
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

fn template_layout(first_templ_source: SourceTempl, attr: AttrData) -> Result<TokenStream> {
    let path = first_templ_source.resolve_path();
    let source = first_templ_source.resolve_source()?;

    let include_source = generate::include_str_source(path.as_deref());
    let (Template { layout, statics, .. }, body) = generate::template(&source, &attr)?;
    let sources = generate::sources(path.as_deref(), &attr.reload, &statics);

    if layout.is_none() {
        return Ok(quote! {{
            #include_source
            let layout_inner = &*self;
            #(#sources)*
            #body
            Ok(())
        }});
    }

    // Nested Layout

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
                #body
                Ok(())
            }
        }
    };

    while let Some(layout_source) = layout.take() {
        counter += 1;
        let path = layout_source.resolve_path();
        let source = layout_source.resolve_source()?;

        let include_source = generate::include_str_source(path.as_deref());
        let (Template { layout: l1, statics, .. }, body) = generate::template(&source, &attr)?;
        let sources = generate::sources(path.as_deref(), &attr.reload, &statics);

        let name = format_ident!("InnerLayout{counter}");
        inner = quote! {
            struct #name<S>(S);

            impl<S> #TemplDisplay for #name<S> where S: #TemplDisplay {
                fn display(&self, writer: &mut impl #TemplWrite) -> ::tour::Result<()> {
                    #inner

                    #include_source
                    let layout_inner = #name_inner(&self.0);
                    #(#sources)*
                    #body
                    Ok(())
                }
            }
        };

        name_inner = name;
        layout = l1;
    }

    Ok(quote! {{
        #inner
        #TemplDisplay::display(&#name_inner(self), writer)
    }})
}

mod generate {
    //! contains functions to generate code step by step
    //!
    //! it is splitted because root template and layout template have different step
    //!
    //! 1. include_str_source
    //! 2. template
    //! 3. sources

    use super::*;

    /// Generate `include_str!("")`
    pub fn include_str_source(path: Option<&str>) -> Option<TokenStream> {
        path.map(|path|quote!{const _: &str = include_str!(#path);})
    }

    pub fn template(source: &str, attr: &AttrData) -> Result<(Template,TokenStream)> {
        let templ = match Parser::new(source, SynVisitor::new()).parse() {
            Ok(ok) => ok.finish(),
            Err(ParseError::Generic(err)) => error!("{err}"),
        };
        let body = codegen::generate(attr, &templ)?;
        Ok((templ,body))
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

