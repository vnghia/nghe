mod handler;

use convert_case::{Case, Casing};
pub use handler::Handler;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{Error, parse_quote};

use crate::endpoint::Attribute;

#[derive(Debug, deluxe::ParseMetaItem)]
struct BuildRouter {
    modules: Vec<syn::Meta>,
    #[deluxe(default = false)]
    filesystem: bool,
    #[deluxe(default = vec![])]
    extensions: Vec<syn::Path>,
}

pub fn build_router(item: TokenStream) -> Result<TokenStream, Error> {
    let input = deluxe::parse2::<BuildRouter>(item)?;
    let endpoints: Vec<_> = input
        .modules
        .into_iter()
        .map(|meta| {
            let module = meta
                .path()
                .get_ident()
                .ok_or_else(|| Error::new(meta.span(), "Meta path ident is missing"))?
                .to_owned();
            let attribute = if let syn::Meta::List(syn::MetaList { ref tokens, .. }) = meta {
                deluxe::parse2(tokens.clone())?
            } else {
                Attribute::builder().build()
            };

            let mut routers = vec![];

            if attribute.form() {
                let form_handler = quote! { #module::form_handler };

                let request = quote! { <#module::Request as nghe_api::common::FormURL> };
                routers.push(quote! {
                    route(
                        #request::URL_FORM,
                        axum::routing::get(#form_handler).post(#form_handler)
                    )
                });
                routers.push(quote! {
                    route(
                        #request::URL_FORM_VIEW,
                        axum::routing::get(#form_handler).post(#form_handler)
                    )
                });
            }

            if attribute.binary() {
                let binary_handler = quote! { #module::binary_handler };

                let request = quote! { <#module::Request as nghe_api::common::BinaryURL> };
                routers.push(quote! {
                    route(
                        #request::URL_BINARY,
                        axum::routing::post(#binary_handler)
                    )
                });
            }

            if attribute.json() {
                let json_handler = quote! { #module::json_handler };

                let request = quote! { <#module::Request as nghe_api::common::JsonURL> };
                routers.push(quote! {
                    route(
                        #request::URL_JSON,
                        axum::routing::post(#json_handler)
                    )
                });
            }

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

    for extension in input.extensions {
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
