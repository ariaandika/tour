//! Code generation.
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::*;

use crate::{
    common::{TemplWrite, path},
    config::Config,
    data::Template,
    file::File,
    metadata::Metadata,
};

mod body;
mod sizehint;

pub fn derive(input: &DeriveInput) -> Result<TokenStream> {
    let conf = Config::default();
    let meta = Metadata::from_attrs(&input.attrs, &conf)?;
    let file = File::from_meta(&meta)?;
    let templ = Template::new(input.ident.clone(), meta, file)?;
    let mut root = quote! { const _: () = };

    brace(&mut root, |tokens| {
        generate_templ(&templ, input, tokens);
    });

    <Token![;]>::default().to_tokens(&mut root);

    Ok(root)
}

pub fn generate_templ(templ: &Template, input: &DeriveInput, root: &mut TokenStream) {
    let ident = &input.ident;
    let (g1, g2, g3) = input.generics.split_for_impl();

    // ===== trait Template =====

    root.extend(quote! {
        #[automatically_derived]
        impl #g1 ::tour::Template for #ident #g2 #g3
    });

    brace(root, |trait_tokens| {

        // ===== render_into() =====

        trait_tokens.extend(quote! {
            fn render_into(&self, writer: &mut impl #TemplWrite) -> ::tour::Result<()>
        });

        brace(trait_tokens, |render_into| {
            body::Visitor::generate(templ, input, render_into);
        });

        // ===== render_block_into() =====

        trait_tokens.extend(quote! {
            fn render_block_into(&self, block: &str, writer: &mut impl #TemplWrite) -> ::tour::Result<()>
        });

        brace(trait_tokens, |tokens| {
            let blocks = templ.file().blocks();

            if blocks.is_empty() {
                tokens.extend(quote! { Err(::tour::Error::NoBlock) });
                return;
            }

            tokens.extend(quote! { match block });
            brace(tokens, |tokens| {
                for block in blocks {
                    let name = &block.templ.name;
                    let name_str = name.to_string();
                    tokens.extend(quote! { #name_str => });
                    brace(tokens, |tokens| {
                        body::Visitor::generate_block(templ, name, input, tokens);
                    });
                }
                tokens.extend(quote! { _ => Err(::tour::Error::NoBlock), });
            });
        });

        // ===== contains_block() =====

        trait_tokens.extend(quote! {
            fn contains_block(&self, block: &str) -> bool
        });

        brace(trait_tokens, |tokens| {
            let blocks = templ.file().blocks();

            if blocks.is_empty() {
                false.to_tokens(tokens);
                return;
            }

            let blocks = blocks
                .iter()
                .map(|block|{
                    block.templ.name.to_string()
                });

            tokens.extend(quote! {
                matches!(block, #(#blocks)|*)
            });
        });

        // ===== size_hint() =====

        trait_tokens.extend(quote! {
            fn size_hint(&self) -> (usize,Option<usize>)
        });

        brace(trait_tokens, |tokens| {
            sizehint::Visitor::new(templ).generate(tokens);
        });

        // ===== size_hint_block() =====

        trait_tokens.extend(quote! {
            fn size_hint_block(&self, block: &str) -> (usize,Option<usize>)
        });

        brace(trait_tokens, |tokens| {
            let blocks = templ.file().blocks();
            if blocks.is_empty() {
                tokens.extend(quote! { (0,None) });
                return;
            }

            tokens.extend(quote! { match block });
            brace(tokens, |tokens| {
                for block in blocks {
                    let name = &block.templ.name;
                    let name_str = name.to_string();
                    tokens.extend(quote! { #name_str => });
                    brace(tokens, |tokens| {
                        sizehint::Visitor::new(templ)
                            .generate_block(&block.templ.name, tokens);
                    });
                }
                tokens.extend(quote! { _ => (0,None), });
            });
        });
    });

    // ===== trait TemplDisplay =====

    root.extend(quote! {
        #[automatically_derived]
        impl #g1 ::tour::TemplDisplay for #ident #g2 #g3 {
            fn display(&self, f: &mut impl ::tour::TemplWrite) -> ::tour::Result<()> {
                self.render_into(f)
            }
        }
    });

    // ===== imports =====

    for import in templ.file().imports() {
        let name = import.generate_name();
        root.extend(quote! {
            struct #name<'a>(&'a #ident);
        });

        // TODO: maybe create new data type contains reference only
        let mut generics = input.generics.clone();
        generics.params.push(syn::parse_quote!('a));
        let input = DeriveInput {
            attrs: vec![],
            vis: Visibility::Inherited,
            ident: name,
            generics,
            data: input.data.clone(),
        };
        generate_templ(import.templ(), &input, root);
    }

    {
        let cwd = templ.meta().path();
        if std::path::Path::new(cwd).is_file() {
            root.extend(quote! {
                const _: &str = include_str!(#cwd);
            });
        }

        for import in templ.file().imports() {
            let path = import.templ().meta().path();
            let path = path::resolve_at(path, cwd);
            let path = path.as_ref();
            if std::path::Path::new(path).is_file() {
                root.extend(quote! {
                    const _: &str = include_str!(#path);
                });
            }
        }
    }
}

fn brace<F>(tokens: &mut TokenStream, call: F)
where
    F: FnOnce(&mut TokenStream)
{
    token::Brace::default().surround(tokens, call);
}

fn paren<F>(tokens: &mut TokenStream, call: F)
where
    F: FnOnce(&mut TokenStream)
{
    token::Paren::default().surround(tokens, call);
}

