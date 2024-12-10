#![allow(clippy::too_many_lines)]
#![feature(iterator_try_collect)]
#![feature(let_chains)]
#![feature(if_let_guard)]
#![feature(proc_macro_span)]

use proc_macro::TokenStream;

mod api;
mod backend;
mod endpoint;
mod orm;

trait IntoTokenStream {
    fn into_token_stream(self) -> TokenStream;
}

impl IntoTokenStream for syn::Error {
    fn into_token_stream(self) -> TokenStream {
        self.to_compile_error().into()
    }
}

impl IntoTokenStream for Result<proc_macro2::TokenStream, syn::Error> {
    fn into_token_stream(self) -> TokenStream {
        match self {
            Ok(r) => r.into(),
            Err(e) => e.into_token_stream(),
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
    match backend::Handler::new(attr.into(), item.into()) {
        Ok(handler) => handler.build().into_token_stream(),
        Err(error) => error.into_token_stream(),
    }
}

#[proc_macro]
pub fn build_router(item: TokenStream) -> TokenStream {
    backend::build_router(item.into()).into_token_stream()
}

#[proc_macro_attribute]
pub fn check_music_folder(attr: TokenStream, item: TokenStream) -> TokenStream {
    orm::check_music_folder(attr.into(), item.into()).into_token_stream()
}
