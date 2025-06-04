//! `Template` derive macro
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::*;
use tour_core::{ParseError, Parser};

use crate::{
    attribute::{AttrData, AttrField, FmtTempl},
    codegen,
    data::{File, Metadata, Template},
    shared::{SourceTempl, TemplDisplay, TemplWrite, error},
    syntax::LayoutTempl,
    visitor::SynVisitor,
};

/// parse input -> codegen
///
/// Inputs:
///
/// - [`AttrData`]: derive macro attribute
/// - [`SourceTempl`]: layout declaration
/// - [`Template`]: template source code
///
/// all function should accept the whole either input type
///
/// ```custom
/// AttrData -> (Metadata,SourceTempl) -> (Metadata,File) -> Template
///
/// LayoutTempl -> (Metadata,SourceTempl) -> (Metadata,File) -> Template
/// ```
pub fn template(input: DeriveInput) -> Result<TokenStream> {
    let DeriveInput { attrs, vis: _, ident, generics, data } = input;

    let displays = genutil::field_display(&data)?;

    // ===== parse input =====

    let attr = AttrData::from_attr(&attrs)?;
    attr.source().validate(&ident)?;
    let meta = Metadata::from_attr(&attr);
    let file = generate::file(attr.source())?;
    let templ = Template::new(meta, file);

    // ===== codegen =====

    let body = codegen::generate(&templ)?;
    let include_source = generate::include_str_source(&templ);
    let sources = generate::sources(&templ);
    let main = quote! {
        #displays
        #include_source
        #(#sources)*
        #body
        Ok(())
    };

    let size_hint = generate::size_hint(&templ)?;

    let (main,expr) = match templ.into_layout() {
        Some(layout) => {
            let main_name = format_ident!("Main{ident}");
            let destructor = genutil::destructor(&data, &ident, quote! { &self.0 });

            let (layouts,names) = template_layout(layout, attr)?;
            let fold = [main_name.clone()]
                .into_iter()
                .chain(names)
                .fold(quote!(&self), |acc, n| quote!(#n(#acc)));

            let main = quote! {
                struct #main_name<'a>(&'a #ident);

                #[automatically_derived]
                impl #TemplDisplay for #main_name<'_> {
                    fn display(&self, writer: &mut impl #TemplWrite) -> ::tour::Result<()> {
                        #destructor
                        #main
                    }
                }

                #layouts
            };

            (main,quote! { #TemplDisplay::display(&#fold, writer) })
        },
        None => {
            let destructor = genutil::destructor(&data, quote! { Self }, quote! { self });

            (quote! { }, quote! { #destructor #main })
        },
    };

    let (g1,g2,g3) = generics.split_for_impl();

    Ok(quote! {
        const _: () = {
            #[automatically_derived]
            impl #g1 ::tour::Template for #ident #g2 #g3 {
                fn render_into(&self, writer: &mut impl #TemplWrite) -> ::tour::Result<()> {
                    #expr
                }

                #size_hint
            }

            #[automatically_derived]
            impl #g1 #TemplDisplay for #ident #g2 #g3 {
                fn display(&self, f: &mut impl #TemplWrite) -> ::tour::Result<()> {
                    self.render_into(f)
                }
            }

            #main
        };
    })
}

/// Returns `(layout,generated layout names)`
fn template_layout(source: LayoutTempl, attr: AttrData) -> Result<(TokenStream, Vec<Ident>)> {
    struct Visitor<'a> {
        names: &'a mut Vec<Ident>,
        counter: usize,
        attr: AttrData,
    }

    impl Visitor<'_> {
        fn generate_name(&mut self, templ: &Template) -> Ident {
            self.counter += 1;
            let suffix = match templ.path() {
                Some(path) => std::path::Path::new(path)
                    .file_stem()
                    .and_then(|e|e.to_str())
                    .unwrap_or("OsFile"),
                None => "Inline",
            };
            let name = format_ident!("L{}{suffix}",self.counter);
            self.names.push(name.clone());
            name
        }

        fn visit_layout(mut self, layout: LayoutTempl) -> Result<TokenStream> {
            // ===== parse input =====

            let source = SourceTempl::from_layout(&layout);
            source.validate(&layout.path)?;
            let meta = Metadata::from_layout(layout, self.attr.reload().clone());
            let file = generate::file(&source)?;
            let templ = Template::new(meta, file);

            // ===== codegen =====

            let body = codegen::generate(&templ)?;
            let include_source = generate::include_str_source(&templ);
            let sources = generate::sources(&templ);
            let body = quote! {
                #include_source
                #(#sources)*
                #body
                Ok(())
            };

            let name = self.generate_name(&templ);
            let nested_layout = match templ.into_layout() {
                Some(layout) => self.visit_layout(layout)?,
                None => quote! { },
            };

            Ok(quote! {
                struct #name<S>(S);

                #[automatically_derived]
                impl<S: #TemplDisplay> #TemplDisplay for #name<S> {
                    fn display(&self, writer: &mut impl #TemplWrite) -> ::tour::Result<()> {
                        #body
                    }
                }

                #nested_layout
            })
        }
    }

    let mut names = vec![];

    let visitor = Visitor { names: &mut names, counter: 0, attr };
    let layout = visitor.visit_layout(source)?;

    Ok((layout,names))
}

mod genutil {
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

    /// destruct fields for convenient
    pub fn destructor(data: &Data, ty: impl ToTokens, me: impl ToTokens) -> TokenStream {
        match data {
            Data::Struct(data) if matches!(data.fields.members().next(),Some(Member::Named(_))) => {
                let fields = data.fields.iter().map(|f|f.ident.as_ref().expect("checked in if guard"));
                quote! { let #ty { #(#fields),* } = #me; }
            }
            // named fields only
            _ => quote! {}
        }
    }
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

    pub fn file(source: &SourceTempl) -> Result<File> {
        match Parser::new(source.resolve_source()?.as_ref(), SynVisitor::new()).parse() {
            Ok(ok) => Ok(ok.finish()),
            Err(ParseError::Generic(err)) => error!("{err}"),
        }
    }

    pub fn size_hint(templ: &Template) -> Result<TokenStream> {
        let (min,max) = crate::sizehint::size_hint(templ)?;
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

    pub fn include_str_source(templ: &Template) -> TokenStream {
        match templ.path() {
            Some(path) => quote! { const _: &str = include_str!(#path); },
            None => quote! { },
        }
    }

    pub fn sources(templ: &Template) -> [TokenStream;2] {
        let path = templ.path();
        let statics = templ.statics();
        match (path.is_some(), templ.reload().as_bool()) {
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
}

