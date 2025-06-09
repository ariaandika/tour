use proc_macro2::TokenStream;
use quote::quote;
use syn::{token::Brace, *};

use crate::{common::TemplWrite, config::Config, data::Template, file::File, metadata::Metadata};

mod body;
mod sizehint;

pub fn generate(input: &DeriveInput) -> Result<TokenStream> {
    let ident = &input.ident;
    let (g1, g2, g3) = input.generics.split_for_impl();

    let conf = Config::default();
    let meta = Metadata::from_attrs(&input.attrs, &conf)?;
    let file = File::from_meta(&meta)?;
    let templ = Template::new(meta, file)?;

    let mut root = quote! { const _: () = };

    brace(&mut root, |root| {
        root.extend(quote! {
            #[automatically_derived]
            impl #g1 ::tour::Template for #ident #g2 #g3
        });

        brace(root, |trait_tokens| {
            trait_tokens.extend(quote! {
                fn render_into(&self, writer: &mut impl #TemplWrite) -> ::tour::Result<()>
            });

            brace(trait_tokens, |render_into| {
                body::Visitor::generate(&templ, input, render_into);
            });

            trait_tokens.extend(quote! {
                fn size_hint(&self) -> (usize,Option<usize>)
            });

            brace(trait_tokens, |render_into| {
                let size_hint = sizehint::Visitor::generate(&templ);
                println!("{size_hint:?}");
            });

            todo!()
        });
    });

    Ok(root)
}

fn brace<F>(root: &mut TokenStream, call: F)
where
    F: FnOnce(&mut TokenStream)
{
    Brace::default().surround(root, call);
}

