use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Error;

#[derive(deluxe::ParseMetaItem)]
struct Handler {
    #[deluxe(default = "trace".into())]
    ret_level: String,
}

pub fn handler(attr: TokenStream, item: TokenStream) -> Result<TokenStream, Error> {
    let handler: syn::ItemFn = syn::parse2(item)?;
    let Handler { ret_level } = deluxe::parse2(attr)?;

    let handler_ident = &handler.sig.ident;
    let json_handler_ident = format_ident!("json_{}", &handler_ident);

    Ok(quote! {
        #[tracing::instrument(skip(database), ret(level = #ret_level), err)]
        #handler

        use axum::extract::State;
        use nghe_api::common::SubsonicResponse;

        pub async fn #json_handler_ident(
            State(app): State<crate::app::state::App>,
            request: Request,
        ) -> Result<axum::Json<SubsonicResponse<Response>>, Error> {
            let response = #handler_ident(&app.database, request).await?;
            Ok(axum::Json(SubsonicResponse::new(response)))
        }
    })
}
