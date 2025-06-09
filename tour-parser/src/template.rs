//! `Template` derive macro
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{Data, DeriveInput, Ident, Result};

use crate::{
    codegen,
    common::{TemplDisplay, TemplWrite},
    config::Config,
    data::Template,
    file::File,
    metadata::Metadata,
    sizehint::{self, SizeHint},
    syntax::LayoutTempl,
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

    let meta = Metadata::from_attrs(&attrs, &conf)?;
    let file = File::from_meta(&meta)?;
    let templ = Template::new(meta, file);

    // ===== codegen =====

    let body = codegen::generate(&templ)?;
    let include = generate::include_str_source(&templ);
    let sources = generate::sources(&templ);
    let main = quote! {
        #include
        #(#sources)*
        #body
        Ok(())
    };

    let imports = {
        let mut imports = quote! {};
        for import in templ.imports() {
            let templ = import.templ();
            let body = codegen::generate(templ)?;
            let include = generate::include_str_source(templ);
            let sources = generate::sources(templ);
            let name = import.generate_name();
            let body = quote! {
                #include
                #(#sources)*
                #body
                Ok(())
            };
            imports.extend(codegen::generate_typed_template(name, quote! { &'a #ident }, body));
        }
        imports
    };
    let blocks = BlockVisitor::generate(&templ)?;
    let mut size_hint = sizehint::size_hint(&templ)?;
    let (meta,file) = templ.into_parts();

    let (main,expr) = match file.into_layout() {
        Some(layout) => {
            let main_name = format_ident!("Main{ident}");
            let destructor = genutil::destructor(&data, &ident, quote! { &self.0 });

            let (layouts, visitor) = LayoutVisitor::generate(layout, meta, &main_name)?;
            let fold = [main_name.clone()]
                .into_iter()
                .chain(visitor.names)
                .fold(quote!(self), |acc, n| quote!(#n(#acc)));
            let body = quote! {
                #destructor
                #main
            };

            size_hint = sizehint::add_size_hint(size_hint, visitor.size_hint);

            let mut main = codegen::generate_typed_template(main_name, quote! { &'a #ident }, body);
            main.extend(layouts);

            (main, quote! { #TemplDisplay::display(&#fold, writer) })
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

            #imports
        };
    })
}

struct BlockVisitor<'a> {
    /// fn render_block()
    blocks: TokenStream,
    /// fn contains_block()
    contains: Vec<String>,
    /// fn size_hint_block()
    size_hints: TokenStream,
    templ: &'a Template,
}

impl BlockVisitor<'_> {
    fn generate(templ: &Template) -> Result<TokenStream> {
        let mut me = BlockVisitor {
            blocks: quote! {},
            contains: vec! {},
            size_hints: quote! {},
            templ,
        };

        for block in templ.blocks().iter().filter(|e|e.templ.pub_token.is_some()) {
            me.visit_block(block)?;
        }

        let BlockVisitor { blocks, contains, size_hints, .. } = me;
        let mut tokens = TokenStream::new();

        if !blocks.is_empty() {
            tokens.extend(quote! {
                fn render_block_into(&self, block: &str, writer: &mut impl #TemplWrite) -> ::tour::Result<()> {
                    match block {
                        #blocks
                        _ => Err(::tour::Error::NoBlock),
                    }
                }
            });
        }

        if !contains.is_empty() {
            tokens.extend(quote! {
                fn contains_block(&self, block: &str) -> bool {
                    matches!(block, #(#contains)|*)
                }
            });
        };

        if !size_hints.is_empty() {
            tokens.extend(quote! {
                fn size_hint_block(&self, block: &str) -> (usize,Option<usize>) {
                    match block {
                        #size_hints
                        _ => (0,None)
                    }
                }
            });
        };

        Ok(tokens)
    }

    fn visit_block(&mut self, block: &BlockContent) -> Result<()> {
        // ===== codegen =====

        let name = block.templ.name.to_string();
        let body = codegen::generate_block(self.templ, &block.templ.name)?;
        let sources = generate::sources(self.templ);
        let size_hint = genutil::size_hint(sizehint::size_hint_block(self.templ, &block.templ.name)?);

        self.blocks.extend(quote! {
            #name => {
                #(#sources)*
                #body
                Ok(())
            },
        });

        self.size_hints.extend(quote! {
            #name => #size_hint,
        });

        self.contains.push(name);

        Ok(())
    }
}

struct LayoutVisitor {
    names: Vec<Ident>,
    size_hint: SizeHint,
    counter: usize,
    meta: Metadata,
}

impl LayoutVisitor {
    fn generate(source: LayoutTempl, meta: Metadata, inner: &Ident) -> Result<(TokenStream, Self)> {
        let mut visitor = LayoutVisitor {
            names: vec![],
            size_hint: (0, None),
            counter: 0,
            meta,
        };
        let layout = visitor.visit_layout(source, inner)?;

        Ok((layout, visitor))
    }

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
        let file = File::from_meta(&meta)?;
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
        let mut template = codegen::generate_typed_template(&name, quote! { #inner<'a> }, &body);

        if let Some(layout) = templ.into_layout() {
            self.visit_layout(layout, &name)?.to_tokens(&mut template);
        };

        Ok(template)
    }
}

mod genutil {
    use syn::Member;

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

