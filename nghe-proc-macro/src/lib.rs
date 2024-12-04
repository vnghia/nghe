#![feature(iterator_try_collect)]
#![feature(let_chains)]
#![feature(proc_macro_span)]

use proc_macro::TokenStream;

mod api;
mod backend;
mod endpoint;
mod orm;

trait IntoTokenStream {
    fn into_token_stream(self) -> TokenStream;
}

impl IntoTokenStream for Result<proc_macro2::TokenStream, syn::Error> {
    fn into_token_stream(self) -> TokenStream {
        match self {
            Ok(r) => r.into(),
            Err(e) => e.to_compile_error().into(),
        }
    }
}

#[proc_macro_derive(Endpoint, attributes(endpoint))]
pub fn derive_endpoint(item: TokenStream) -> TokenStream {
    api::derive_endpoint(item.into()).into_token_stream()
}

#[proc_macro_attribute]
pub fn api_derive(attr: TokenStream, item: TokenStream) -> TokenStream {
    api::derive(attr.into(), item.into()).into_token_stream()
}

#[proc_macro_attribute]
pub fn handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    backend::handler(attr.into(), item.into()).into_token_stream()
}

#[proc_macro]
pub fn build_router(item: TokenStream) -> TokenStream {
    backend::build_router(item.into()).into_token_stream()
}

#[proc_macro_attribute]
pub fn check_music_folder(attr: TokenStream, item: TokenStream) -> TokenStream {
    orm::check_music_folder(attr.into(), item.into()).into_token_stream()
}
