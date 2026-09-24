mod handler;

use std::ops::Deref;

use convert_case::{Case, Casing};
pub use handler::Handler;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Error, parse_quote};

use crate::utils;

#[derive(Debug, darling::FromMeta)]
#[darling(derive_syn_parse)]
struct BuildRouter {
    modules: utils::syn::Vec<syn::Ident>,
    #[darling(default = || false)]
    filesystem: bool,
    #[darling(default)]
    extensions: utils::syn::Vec<syn::Path>,
}

pub fn build_router(item: TokenStream) -> Result<TokenStream, Error> {
    let input: BuildRouter = syn::parse2(item)?;
    let endpoints: Vec<_> = input
        .modules
        .deref()
        .iter()
        .map(|module| {
            let mut routers = vec![];

            let request_handler = quote! { #module::request_handler };
            let request = quote! { <#module::Request as nghe_api::common::EndpointURL> };
            routers.push(quote! {
                route(
                    #request::URL,
                    axum::routing::any(#request_handler)
                )
            });
            routers.push(quote! {
                route(
                    #request::URL_VIEW,
                    axum::routing::any(#request_handler)
                )
            });

            Ok::<_, Error>(routers)
        })
        .try_collect::<Vec<_>>()?
        .into_iter()
        .flatten()
        .collect();

    let mut router_args: Vec<syn::FnArg> = vec![];
    let mut router_layers: Vec<syn::Expr> = vec![];

    if input.filesystem {
        router_args.push(parse_quote!(filesystem: crate::filesystem::Filesystem));
        router_layers.push(parse_quote!(layer(axum::Extension(filesystem))));
    }

    for extension in &*input.extensions {
        let arg = extension
            .segments
            .iter()
            .map(|segment| segment.ident.to_string().to_case(Case::Snake))
            .collect::<Vec<_>>()
            .join("_");
        let arg = format_ident!("{arg}");
        router_args.push(parse_quote!(#arg: #extension));
        router_layers.push(parse_quote!(layer(axum::Extension(#arg))));
    }

    let router_body: syn::Expr = if router_layers.is_empty() {
        parse_quote!(axum::Router::new().#( #endpoints ).*)
    } else {
        parse_quote!(axum::Router::new().#( #endpoints ).*.#( #router_layers ).*)
    };

    Ok(quote! {
        #[coverage(off)]
        pub fn router(#( #router_args ),*) -> axum::Router<crate::database::Database> {
            #router_body
        }
    })
}
