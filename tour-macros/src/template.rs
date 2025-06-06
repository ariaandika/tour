//! `Template` derive macro
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::*;

use crate::{
    attribute,
    codegen,
    config::Config,
    data::{Metadata, Template},
    shared::{TemplDisplay, TemplWrite},
    sizehint::{self, SizeHint},
    syntax::LayoutTempl, visitor,
};

/// parse input -> codegen
///
/// Inputs:
///
/// - [`AttrData`]: derive macro attribute
/// - [`Source`]: layout declaration
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
    let conf = Config::default();

    // ===== parse input =====

    let meta = attribute::generate_meta(&attrs, &conf)?;
    let file = visitor::generate_file(&meta)?;
    let templ = Template::new(meta, file);

    // ===== codegen =====

    let body = codegen::generate(&templ)?;
    let include_source = generate::include_str_source(&templ);
    let sources = generate::sources(&templ);
    let main = quote! {
        #include_source
        #(#sources)*
        #body
        Ok(())
    };

    let blocks = template_block(&templ)?;
    let mut size_hint = sizehint::size_hint(&templ)?;
    let (meta,file) = templ.into_parts();

    let (main,expr) = match file.into_layout() {
        Some(layout) => {
            let main_name = format_ident!("Main{ident}");
            let destructor = genutil::destructor(&data, &ident, quote! { &self.0 });

            let (layouts, visitor) = template_layout(layout, meta, &main_name)?;
            let fold = [main_name.clone()]
                .into_iter()
                .chain(visitor.names)
                .fold(quote!(self), |acc, n| quote!(#n(#acc)));

            size_hint = sizehint::add_size_hint(size_hint, visitor.size_hint);

            let main = quote! {
                struct #main_name<'a>(&'a #ident);

                #[automatically_derived]
                impl std::ops::Deref for #main_name<'_> {
                    type Target = #ident;

                    fn deref(&self) -> &Self::Target {
                        self.0
                    }
                }

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

    let size_hint = genutil::size_hint(size_hint);
    let (g1,g2,g3) = generics.split_for_impl();

    Ok(quote! {
        const _: () = {
            #[automatically_derived]
            impl #g1 ::tour::Template for #ident #g2 #g3 {
                fn render_into(&self, writer: &mut impl #TemplWrite) -> ::tour::Result<()> {
                    #expr
                }

                fn size_hint(&self) -> (usize,Option<usize>) {
                    #size_hint
                }

                #blocks
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

fn template_block(templ: &Template) -> Result<TokenStream> {
    let mut blocks = quote! { };
    let mut contains = vec! { };
    let mut size_hints = quote! { };

    for block in templ.blocks().iter().filter(|e|e.templ.pub_token.is_some()) {
        // ===== codegen =====

        let name = block.templ.name.to_string();
        let body = codegen::generate_block(templ, &block.templ.name)?;
        let sources = generate::sources(templ);
        blocks.extend(quote! {
            #name => {
                #(#sources)*
                #body
                Ok(())
            },
        });

        let size_hint = genutil::size_hint(sizehint::size_hint_block(templ, &block.templ.name)?);
        size_hints.extend(quote! {
            #name => #size_hint,
        });

        contains.push(name);
    }

    let blocks = match blocks.is_empty() {
        true => quote! { },
        false => quote! {
            fn render_block_into(&self, block: &str, writer: &mut impl #TemplWrite) -> ::tour::Result<()> {
                match block {
                    #blocks
                    _ => Err(::tour::Error::NoBlock),
                }
            }
        }
    };

    let contains = match contains.is_empty() {
        true => quote! { },
        false => quote! {
            fn contains_block(&self, block: &str) -> bool {
                matches!(block, #(#contains)|*)
            }
        }
    };

    let size_hint = match size_hints.is_empty() {
        true => quote! { },
        false => quote! {
            fn size_hint_block(&self, block: &str) -> (usize,Option<usize>) {
                match block {
                    #size_hints
                    _ => (0,None)
                }
            }
        }
    };

    Ok(quote! {
        #blocks
        #contains
        #size_hint
    })
}

/// Returns `(layout,generated layout names)`
fn template_layout(source: LayoutTempl, meta: Metadata, inner: &Ident) -> Result<(TokenStream, LayoutVisitor)> {
    let mut visitor = LayoutVisitor {
        names: vec![],
        size_hint: (0, None),
        counter: 0,
        meta,
    };
    let layout = visitor.visit_layout(source, inner)?;

    Ok((layout, visitor))
}

struct LayoutVisitor {
    names: Vec<Ident>,
    size_hint: SizeHint,
    counter: usize,
    meta: Metadata,
}

impl LayoutVisitor {
    fn generate_name(&mut self, templ: &Template) -> Ident {
        self.counter += 1;
        let suffix = std::path::Path::new(templ.path())
            .file_stem()
            .and_then(|e|e.to_str())
            .unwrap_or("OsFile");
        let name = format_ident!("L{}{suffix}",self.counter);
        self.names.push(name.clone());
        name
    }

    fn visit_layout(&mut self, layout: LayoutTempl, inner: &Ident) -> Result<TokenStream> {
        // ===== parse input =====

        let meta = self.meta.clone_with_layout(&layout);
        let file = visitor::generate_file(&meta)?;
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

        let size_hint = sizehint::size_hint(&templ)?;
        self.size_hint = sizehint::add_size_hint(self.size_hint, size_hint);

        let name = self.generate_name(&templ);
        let nested_layout = match templ.into_layout() {
            Some(layout) => self.visit_layout(layout, &name)?,
            None => quote! { },
        };

        Ok(quote! {
            struct #name<'a>(#inner<'a>);

            #[automatically_derived]
            impl<'a> std::ops::Deref for #name<'a> {
                type Target = #inner<'a>;

                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }

            #[automatically_derived]
            impl #TemplDisplay for #name<'_> {
                fn display(&self, writer: &mut impl #TemplWrite) -> ::tour::Result<()> {
                    #body
                }
            }

            #nested_layout
        })
    }
}

mod genutil {
    use super::*;

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

    pub fn size_hint((min, max): SizeHint) -> TokenStream {
        let max = match max {
            Some(max) => quote! { Some(#max) },
            None => quote! { None },
        };
        quote! {
            (#min,#max)
        }
    }
}

mod generate {
    use super::*;

    pub fn include_str_source(templ: &Template) -> TokenStream {
        let path = templ.path();
        match std::path::Path::new(path).is_file() {
            true => quote! { const _: &str = include_str!(#path); },
            false => quote! {},
        }
    }

    pub fn sources(templ: &Template) -> [TokenStream;2] {
        let path = templ.path();
        let statics = templ.statics();
        match (templ.is_file(), templ.reload().as_bool()) {
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

