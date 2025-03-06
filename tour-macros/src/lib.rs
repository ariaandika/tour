use proc_macro::TokenStream;
use quote::ToTokens;

mod template;

#[proc_macro]
pub fn render_to(input: TokenStream) -> TokenStream {
    match syn::parse::<template::Template>(input) {
        Ok(ok) => ok.to_token_stream().into(),
        Err(err) => err.into_compile_error().into(),
    }
}

