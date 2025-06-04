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

/// parse input -> codegen
pub fn template(input: DeriveInput) -> Result<TokenStream> {
    let DeriveInput { attrs, vis: _, ident, generics, data } = input;

    // ===== parse input =====

    let attr = AttrData::from_attr(&attrs)?;
    let path = attr.resolve_path();

    // ===== inherited parse input =====

    let (templ,body) = generate::template(attr.resolve_source()?, &attr)?;

    // ===== codegen =====

    let destructor = generate::destructor(&ident, &data);
    let displays = generate::field_display(&data)?;

    let size_hint = generate::size_hint(&attr, &templ)?;
    let include_source = generate::include_str_source(path.as_deref());
    let sources = generate::sources(path.as_deref(), attr.reload(), &templ.statics);

    let layout = match templ.layout {
        Some(layout) => {
            let layout = template_layout(layout, attr)?;
            quote! {
                fn render_layout_into(&self, writer: &mut impl #TemplWrite) -> ::tour::Result<()> {
                    #layout
                }
            }
        },
        None => quote! {  },
    };

    let (g1,g2,g3) = generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #g1 ::tour::Template for #ident #g2 #g3 {
            fn render_into(&self, writer: &mut impl #TemplWrite) -> ::tour::Result<()> {
                #destructor
                #displays

                #include_source
                #(#sources)*
                #body
                Ok(())
            }

            #layout

            #size_hint
        }

        #[automatically_derived]
        impl #g1 #TemplDisplay for #ident #g2 #g3 {
            fn display(&self, f: &mut impl #TemplWrite) -> ::tour::Result<()> {
                self.render_into(f)
            }
        }
    })
}

fn template_layout(source: SourceTempl, attr: AttrData) -> Result<TokenStream> {
    struct Visitor<'a> {
        names: &'a mut Vec<Ident>,
        counter: usize,
        attr: AttrData,
    }

    impl Visitor<'_> {
        fn generate_name(&mut self, path: Option<&str>) -> Ident {
            self.counter += 1;
            let suffix = std::path::Path::new(path.unwrap_or("Inline"))
                .file_stem()
                .and_then(|e|e.to_str())
                .unwrap_or("OsFile");
            let name = format_ident!("L{}{suffix}",self.counter);
            self.names.push(name.clone());
            name
        }

        fn visit_layout(mut self, source: SourceTempl) -> Result<TokenStream> {
            let path = source.resolve_path();
            let source = source.resolve_source()?;

            let (Template { layout, statics, .. }, body) = generate::template(&source, &self.attr)?;
            let include_source = generate::include_str_source(path.as_deref());
            let sources = generate::sources(path.as_deref(), self.attr.reload(), &statics);

            let name = self.generate_name(path.as_deref());
            let nested_layout = match layout {
                Some(source) => self.visit_layout(source)?,
                None => quote! { },
            };

            Ok(quote! {
                #nested_layout

                struct #name<S>(S);

                impl<S> #TemplDisplay for #name<S> where S: #TemplDisplay {
                    fn display(&self, writer: &mut impl #TemplWrite) -> ::tour::Result<()> {
                        #include_source
                        #(#sources)*
                        #body
                        Ok(())
                    }
                }
            })
        }
    }

    let mut names = vec![];

    let visitor = Visitor { names: &mut names, counter: 0, attr };
    let inner = visitor.visit_layout(source)?;

    let fold = names.into_iter().fold(syn::parse_quote!(self), |acc,n| -> Expr {
        syn::parse_quote!(#n(#acc))
    });

    Ok(quote! {
        #inner
        #TemplDisplay::display(&#fold, writer)
    })
}

mod generate {
    //! contains functions to generate code step by step
    //!
    //! it is splitted because root template and layout template have different step
    //!
    //! 1. template, template body
    //! 2. include_str_source, `include_str!()` to trigger recompile on template change
    //! 3. sources, static contents as array to allow runtime reload
    use super::*;

    /// fields with `#[fmt(display)]`
    pub fn field_display(data: &Data) -> Result<TokenStream> {
        match data {
            Data::Struct(data) if matches!(data.fields.members().next(),Some(Member::Named(_))) => {
                let mut displays = quote! {};

                for field in &data.fields {
                    let attr = AttrField::from_attr(&field.attrs)?;
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

                Ok(displays)
            }
            // named fields only
            _ => Ok(quote! {})
        }
    }

    pub fn template(source: &str, attr: &AttrData) -> Result<(Template,TokenStream)> {
        let templ = match Parser::new(source, SynVisitor::new()).parse() {
            Ok(ok) => ok.finish(),
            Err(ParseError::Generic(err)) => error!("{err}"),
        };
        let body = codegen::generate(attr, &templ)?;
        Ok((templ,body))
    }

    pub fn size_hint(attr: &AttrData, templ: &Template) -> Result<TokenStream> {
        let (min,max) = crate::sizehint::size_hint(attr, templ)?;
        let max = match max {
            Some(max) => quote! { Some(#max) },
            None => quote! { None },
        };
        Ok(quote! {
            fn size_hint(&self) -> (usize,Option<usize>) {
                (#min,#max)
            }
        })
    }

    pub fn include_str_source(path: Option<&str>) -> Option<TokenStream> {
        path.map(|path|quote!{const _: &str = include_str!(#path);})
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
            (false, _) if statics.is_empty() => <_>::default(),
            (false, _) => [quote!{ let sources = [#(#statics),*]; },<_>::default()],
        }
    }

    /// destruct fields for convenient
    pub fn destructor(ident: &Ident, data: &Data) -> TokenStream {
        match data {
            Data::Struct(data) if matches!(data.fields.members().next(),Some(Member::Named(_))) => {
                let fields = data.fields.iter().map(|f|f.ident.as_ref().expect("checked in if guard"));
                quote! { let #ident { #(#fields),* } = self; }
            }
            // named fields only
            _ => quote! {}
        }
    }
}

