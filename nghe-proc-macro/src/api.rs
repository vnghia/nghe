use concat_string::concat_string;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, Ident};

#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(endpoint))]
struct Endpoint {
    path: String,
    response: Option<Ident>,
}

pub fn derive_endpoint(item: TokenStream) -> Result<TokenStream, Error> {
    let mut input: syn::DeriveInput = syn::parse2(item)?;
    let Endpoint { path, response } = deluxe::extract_attributes(&mut input)?;

    let ident = &input.ident;

    let endpoint = concat_string!("/rest/", &path);
    let endpoint_view = concat_string!("/rest/", &path, ".view");

    let response = response.unwrap_or(quote::format_ident!("Response"));

    Ok(quote! {
        impl crate::common::Endpoint for #ident {
            const ENDPOINT: &'static str = #endpoint;
            const ENDPOINT_VIEW: &'static str = #endpoint_view;

            type Response = #response;
        }
    })
}
