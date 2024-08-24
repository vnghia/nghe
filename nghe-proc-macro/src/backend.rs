use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Error;

#[derive(Debug, deluxe::ParseMetaItem)]
struct Handler {
    #[deluxe(default = "trace".into())]
    ret_level: String,
    role: Option<syn::Ident>,
    #[deluxe(default = true)]
    need_auth: bool,
}

#[derive(Debug, deluxe::ParseMetaItem)]
struct BuildRouter {
    modules: Vec<syn::Ident>,
}

pub fn handler(attr: TokenStream, item: TokenStream) -> Result<TokenStream, Error> {
    let input: syn::ItemFn = syn::parse2(item)?;
    let Handler { ret_level, role, need_auth } = deluxe::parse2(attr)?;

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

        use crate::app::auth::{Authorize, BinaryUser, GetUser, PostUser};

        impl Authorize for Request {
            fn authorize(self, role: crate::orm::users::Role) -> Result<Self, Error> {
                if #authorize_fn {
                    Ok(self)
                } else {
                    Err(Error::MissingRole(#missing_role))
                }
            }
        }

        pub async fn json_get_handler(
            State(app): State<crate::app::state::App>,
            user: GetUser<Request>,
        ) -> Result<axum::Json<SubsonicResponse<<Request as Endpoint>::Response>>, Error> {
            let response = #ident(&app.database, user.request).await?;
            Ok(axum::Json(SubsonicResponse::new(response)))
        }

        pub async fn json_post_handler(
            State(app): State<crate::app::state::App>,
            user: PostUser<Request>,
        ) -> Result<axum::Json<SubsonicResponse<<Request as Endpoint>::Response>>, Error> {
            let response = #ident(&app.database, user.request).await?;
            Ok(axum::Json(SubsonicResponse::new(response)))
        }

        pub async fn binary_handler(
            State(app): State<crate::app::state::App>,
            user: BinaryUser<Request, #need_auth>,
        ) -> Result<Vec<u8>, Error> {
            let response = #ident(&app.database, user.request).await?;
            Ok(bitcode::encode(&response))
        }
    })
}

pub fn build_router(item: TokenStream) -> Result<TokenStream, Error> {
    let endpoints: Vec<_> = deluxe::parse2::<BuildRouter>(item)?
        .modules
        .into_iter()
        .flat_map(|module| {
            let module = format_ident!("{module}");
            let request = quote! { <#module::Request as nghe_api::common::Endpoint> };

            let json_get_handler = quote! { #module::json_get_handler };
            let json_post_handler = quote! { #module::json_post_handler };
            let binary_handler = quote! { #module::binary_handler };

            vec![
                quote! {
                    route(
                        #request::ENDPOINT,
                        axum::routing::get(#json_get_handler).post(#json_post_handler)
                    )
                },
                quote! {
                    route(
                        #request::ENDPOINT_VIEW,
                        axum::routing::get(#json_get_handler).post(#json_post_handler)
                    )
                },
                quote! {
                    route(
                        #request::ENDPOINT_BINARY,
                        axum::routing::post(#binary_handler)
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
