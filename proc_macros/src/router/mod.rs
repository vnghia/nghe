use concat_string::concat_string;
use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Error, Expr};

use crate::expr_to_string;

#[derive(Debug, deluxe::ParseMetaItem)]
#[deluxe(transparent(flatten_unnamed, append))]
struct BuildRouter {
    endpoints: Vec<Expr>,
}

pub fn build_router(input: TokenStream) -> Result<TokenStream, Error> {
    let endpoints = deluxe::parse2::<BuildRouter>(input)?
        .endpoints
        .iter()
        .flat_map(|module| {
            let module_str = expr_to_string(module).unwrap();
            let main_endpoint = module_str.to_case(Case::Camel);
            let endpoint = concat_string!("/rest/", &main_endpoint);
            let endpoint_view = concat_string!("/rest/", &main_endpoint, ".view");

            let handler = format_ident!("{}_handler", module_str);
            let handler_path = quote! { #module::#handler };

            vec![
                quote! {
                    route(#endpoint, axum::routing::get(#handler_path).post(#handler_path))
                },
                quote! {
                    route(#endpoint_view, axum::routing::get(#handler_path).post(#handler_path))
                },
            ]
        })
        .collect::<Vec<_>>();

    Ok(quote! {
        axum::Router::new().#( #endpoints ).*
    })
}
