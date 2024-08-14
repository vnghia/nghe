#![allow(clippy::unnecessary_wraps)]

use proc_macro2::TokenStream;
use quote::quote;
use syn::Error;

pub fn derive_endpoint() -> Result<TokenStream, Error> {
    Ok(quote! {
        impl crate::common::endpoint::Endpoint for Request {
            type Response = Response;
        }
    })
}
