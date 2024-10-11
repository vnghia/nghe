use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{parse_quote, Error};

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
    #[deluxe(default = false)]
    filesystem: bool,
    #[deluxe(default = vec![])]
    extensions: Vec<syn::Path>,
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

    let mut common_args: Vec<syn::FnArg> =
        vec![parse_quote!(extract::State(database): extract::State<crate::database::Database>)];
    let mut pass_args: Vec<syn::Expr> = vec![parse_quote!(&database)];

    for arg in &input.sig.inputs {
        if let syn::FnArg::Typed(arg) = arg
            && let syn::Pat::Ident(pat) = arg.pat.as_ref()
            && pat.ident != "database"
            && pat.ident != "request"
        {
            match arg.ty.as_ref() {
                syn::Type::Path(ty) => {
                    common_args
                        .push(parse_quote!(extract::Extension(#pat): extract::Extension<#ty>));
                    pass_args.push(parse_quote!(#pat));
                }
                syn::Type::Reference(ty) => {
                    let ty = ty.elem.as_ref();
                    common_args
                        .push(parse_quote!(extract::Extension(#pat): extract::Extension<#ty>));
                    pass_args.push(parse_quote!(&#pat));
                }
                _ => {
                    return Err(syn::Error::new(
                        arg.ty.span(),
                        "Only path type and reference type are supported for handler function",
                    ));
                }
            }
        }
    }

    let json_get_args =
        [common_args.as_slice(), [parse_quote!(user: GetUser<Request>)].as_slice()].concat();
    let json_post_args =
        [common_args.as_slice(), [parse_quote!(user: PostUser<Request>)].as_slice()].concat();
    let binary_args =
        [common_args.as_slice(), [parse_quote!(user: BinaryUser<Request, #need_auth>)].as_slice()]
            .concat();
    pass_args.push(parse_quote!(user.request));

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

        use axum::extract;
        use nghe_api::common::{EncodableEndpoint, SubsonicResponse};

        use crate::auth::{Authorize, BinaryUser, GetUser, PostUser};

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
            #( #json_get_args ),*
        ) -> Result<axum::Json<SubsonicResponse<<Request as EncodableEndpoint>::Response>>, Error> {
            let response = #ident(#( #pass_args ),*).await?;
            Ok(axum::Json(SubsonicResponse::new(response)))
        }

        pub async fn json_post_handler(
            #( #json_post_args ),*
        ) -> Result<axum::Json<SubsonicResponse<<Request as EncodableEndpoint>::Response>>, Error> {
            let response = #ident(#( #pass_args ),*).await?;
            Ok(axum::Json(SubsonicResponse::new(response)))
        }

        pub async fn binary_handler(
            #( #binary_args ),*
        ) -> Result<Vec<u8>, Error> {
            let response = #ident(#( #pass_args ),*).await?;
            Ok(bitcode::encode(&response))
        }
    })
}

pub fn build_router(item: TokenStream) -> Result<TokenStream, Error> {
    let input = deluxe::parse2::<BuildRouter>(item)?;
    let endpoints: Vec<_> = input
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
            .map(|segment| segment.ident.to_string().to_lowercase())
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
        pub fn router(#( #router_args ),*) -> axum::Router<crate::database::Database> {
            #router_body
        }
    })
}
