use proc_macro2::TokenStream;
use quote::quote;
use tour_core::Delimiter;

pub fn display(delim: Delimiter, expr: syn::Expr) -> TokenStream {
    use Delimiter::*;

    match delim {
        Quest => quote! {&::tour::Debug(&#expr)},
        Percent => quote! {&::tour::Display(&#expr)},
        Brace | Bang | Hash => quote! {&#expr},
    }
}

pub fn writer(delim: Delimiter) -> TokenStream {
    use Delimiter::*;

    match delim {
        Bang => quote! {&mut *writer},
        Brace | Percent | Quest | Hash => quote! {&mut ::tour::Escape(&mut *writer)},
    }
}

