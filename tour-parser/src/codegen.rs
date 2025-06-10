//! Code generation.
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::*;

use crate::{
    common::{TemplWrite, path},
    config::Config,
    data::Template,
    file::File,
    metadata::{Metadata, TemplKind},
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

fn generate_templ(templ: &Template, input: &DeriveInput, root: &mut TokenStream) {
    let ident = &input.ident;
    let (g1, g2, g3) = input.generics.split_for_impl();

    // ===== trait Template =====

    let cwd = templ.meta().path();
    if std::path::Path::new(cwd).is_file() {
        root.extend(quote! {
            #[doc = concat!(" ",include_str!(#cwd))]
        });
    } else if let Some(src) = templ.meta().inline() {
        root.extend(quote! {
            #[doc = concat!(" ",#src)]
        });
    }

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

        let blocks = templ.file().blocks();
        let prefix = quote! {
            fn render_block_into(&self, block: &str, writer: &mut impl #TemplWrite) -> ::tour::Result<()>
        };

        brace_if(!blocks.is_empty(), prefix, trait_tokens, |tokens| {
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

        let is_ok = matches!(templ.meta().kind(), TemplKind::Main) && !blocks.is_empty();
        let prefix = quote! {
            fn contains_block(&self, block: &str) -> bool
        };

        brace_if(is_ok, prefix, trait_tokens, |tokens| {
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

        let is_skip = !matches!(templ.meta().kind(), TemplKind::Main);
        let size = if is_skip {
            (0,None)
        } else {
            sizehint::Visitor::new(templ).calculate()
        };

        let is_sized = !sizehint::is_empty(size);
        let prefix = quote! {
            fn size_hint(&self) -> (usize,Option<usize>)
        };

        brace_if(is_sized, prefix, trait_tokens, |tokens| {
            sizehint::generate(size, tokens);
        });

        // ===== size_hint_block() =====

        let is_ok = matches!(templ.meta().kind(), TemplKind::Main) && !blocks.is_empty();
        let blocks = if is_ok {
            blocks
                .iter()
                .map(|block|{
                    let block_name = &block.templ.name;
                    (
                        sizehint::Visitor::new(templ).calculate_block(block_name),
                        block.templ.name.to_string()
                    )
                })
                .filter(|e|sizehint::is_empty(e.0))
                .collect()

        } else {
            vec![]
        };

        let is_sized = !blocks.is_empty();
        let prefix = quote! {
            fn size_hint_block(&self, block: &str) -> (usize,Option<usize>)
        };

        brace_if(is_sized, prefix, trait_tokens, |tokens| {
            tokens.extend(quote! { match block });
            brace(tokens, |tokens| {
                for (size,name) in blocks {
                    tokens.extend(quote! { #name => });
                    brace(tokens, |tokens| {
                        sizehint::generate(size, tokens);
                    });
                }
                tokens.extend(quote! { _ => (0,None), });
            });
        });
    });

    // ===== trait TemplDisplay =====

    if matches!(templ.meta().kind(), TemplKind::Main) {
        root.extend(quote! {
            #[automatically_derived]
            impl #g1 ::tour::TemplDisplay for #ident #g2 #g3 {
                fn display(&self, f: &mut impl ::tour::TemplWrite) -> ::tour::Result<()> {
                    self.render_into(f)
                }
            }
        });
    }

    // ===== imports =====

    for import in templ.file().imports() {
        let name = import.alias();
        let path = import
            .templ()
            .meta()
            .path()
            .trim_start_matches(path::cwd().to_str().unwrap_or(""))
            .trim_start_matches("/");
        let doc = if path.is_empty() {
            quote! { }
        } else {
            quote! { #[doc = concat!(" ",#path)] }
        };

        let mut generics = input.generics.clone();
        if !generics.lifetimes().any(|e|e.lifetime.ident=="tour_ref") {
            generics.params.push(syn::parse_quote!('tour_ref));
        }
        let (t1,t2,_) = generics.split_for_impl();

        let input: DeriveInput = syn::parse_quote! {
            #doc
            struct #name #t1 (&'tour_ref #ident #g2) #g3;
        };
        input.to_tokens(root);

        generate_templ(import.templ(), &input, root);

        root.extend(quote! {
            #[automatically_derived]
            impl #t1 ::std::ops::Deref for #name #t2 #g3 {
                type Target = #ident #g2;
                fn deref(&self) -> &Self::Target {
                    self.0
                }
            }
        });
    }
}

fn brace<F>(tokens: &mut TokenStream, call: F)
where
    F: FnOnce(&mut TokenStream)
{
    token::Brace::default().surround(tokens, call);
}

fn brace_if<F>(cond: bool, prefix: impl ToTokens, tokens: &mut TokenStream, call: F)
where
    F: FnOnce(&mut TokenStream)
{
    if cond {
        prefix.to_tokens(tokens);
        token::Brace::default().surround(tokens, call);
    }
}

fn paren<F>(tokens: &mut TokenStream, call: F)
where
    F: FnOnce(&mut TokenStream)
{
    token::Paren::default().surround(tokens, call);
}

