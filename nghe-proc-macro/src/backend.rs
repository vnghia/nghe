use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Error;

#[derive(Debug, deluxe::ParseMetaItem)]
struct Handler {
    #[deluxe(default = "trace".into())]
    ret_level: String,
    role: Option<syn::Ident>,
}

#[derive(Debug, deluxe::ParseMetaItem)]
struct BuildRouter {
    modules: Vec<syn::Ident>,
}

pub fn handler(attr: TokenStream, item: TokenStream) -> Result<TokenStream, Error> {
    let input: syn::ItemFn = syn::parse2(item)?;
    let Handler { ret_level, role } = deluxe::parse2(attr)?;

    let ident = &input.sig.ident;
    if ident != "handler" {
        return Err(syn::Error::new(
            ident.span(),
            "Function derived with `handler` should be named `handler`",
        ));
    }

    let (authorize_fn, missing_role) = if let Some(role) = role {
        if role == "admin" {
            (quote! { role.admin }, role.to_string())
        } else {
            (quote! { role.admin || role.#role }, role.to_string())
        }
    } else {
        (quote! { true }, String::default())
    };

    Ok(quote! {
        #[tracing::instrument(skip(database), ret(level = #ret_level), err)]
        #input

        use axum::extract::State;
        use nghe_api::common::{Endpoint, SubsonicResponse};

        use crate::app::auth::{Authorize, Get};

        impl Authorize for Request {
            fn authorize(self, role: crate::orm::users::Role) -> Result<Self, Error> {
                if #authorize_fn {
                    Ok(self)
                } else {
                    Err(Error::Unauthorized(#missing_role))
                }
            }
        }

        pub async fn json_handler(
            State(app): State<crate::app::state::App>,
            authorized: Get<Request>,
        ) -> Result<axum::Json<SubsonicResponse<<Request as Endpoint>::Response>>, Error> {
            let response = #ident(&app.database, authorized.request).await?;
            Ok(axum::Json(SubsonicResponse::new(response)))
        }
    })
}

pub fn build_router(item: TokenStream) -> Result<TokenStream, Error> {
    let endpoints: Vec<_> = deluxe::parse2::<BuildRouter>(item)?
        .modules
        .into_iter()
        .flat_map(|module| {
            let module = format_ident!("{module}");
            let request = quote! {<#module::Request as nghe_api::common::Endpoint>};
            let json_handler = quote! {#module::json_handler};

            vec![
                quote! {
                    route(
                        #request::ENDPOINT,
                        axum::routing::get(#json_handler).post(#json_handler)
                    )
                },
                quote! {
                    route(
                        #request::ENDPOINT_VIEW,
                        axum::routing::get(#json_handler).post(#json_handler)
                    )
                },
            ]
        })
        .collect();

    Ok(quote! {
        pub fn router() -> axum::Router<crate::app::state::App> {
            axum::Router::new().#( #endpoints ).*
        }
    })
}
