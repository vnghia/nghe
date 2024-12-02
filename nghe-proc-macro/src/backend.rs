#![allow(clippy::too_many_lines)]

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{parse_quote, Error};

use crate::endpoint::Attribute;

#[derive(Debug, deluxe::ParseMetaItem)]
struct Handler {
    #[deluxe(default = "trace".into())]
    ret_level: String,
    role: Option<syn::Ident>,
    #[deluxe(flatten)]
    attribute: Attribute,
    #[deluxe(default = vec![])]
    headers: Vec<syn::Ident>,
    #[deluxe(default = true)]
    need_auth: bool,
}

#[derive(Debug, deluxe::ParseMetaItem)]
struct BuildRouter {
    modules: Vec<syn::Expr>,
    #[deluxe(default = false)]
    filesystem: bool,
    #[deluxe(default = vec![])]
    extensions: Vec<syn::Path>,
}

#[derive(Debug)]
struct ModuleConfig {
    pub ident: syn::Ident,
    pub attribute: Attribute,
}

impl TryFrom<syn::Expr> for ModuleConfig {
    type Error = Error;

    fn try_from(value: syn::Expr) -> Result<Self, Self::Error> {
        let span = value.span();
        match value {
            syn::Expr::Path(syn::ExprPath { path, .. }) => {
                let ident = path
                    .get_ident()
                    .ok_or_else(|| Error::new(span, "Only `Path` with one segment is supported"))?
                    .to_owned();
                Ok(Self { ident, attribute: Attribute::builder().build() })
            }
            syn::Expr::Struct(syn::ExprStruct { path, fields, .. }) => {
                let ident = path
                    .get_ident()
                    .ok_or_else(|| {
                        Error::new(span, "Only `Struct` path with one segment is supported")
                    })?
                    .to_owned();

                let extract_bool = |key: &'static str| -> Option<bool> {
                    fields.iter().find_map(|value| {
                        if let syn::Member::Named(ref ident) = value.member
                            && ident == key
                            && let syn::Expr::Lit(syn::ExprLit { ref lit, .. }) = value.expr
                            && let syn::Lit::Bool(syn::LitBool { value, .. }) = lit
                        {
                            Some(*value)
                        } else {
                            None
                        }
                    })
                };

                Ok(Self {
                    ident,
                    attribute: Attribute::builder()
                        .maybe_internal(extract_bool("internal"))
                        .maybe_binary(extract_bool("binary"))
                        .maybe_json(extract_bool("json"))
                        .build(),
                })
            }
            _ => Err(Error::new(span, "Only `Path` and `Struct` expressions are supported")),
        }
    }
}

pub fn handler(attr: TokenStream, item: TokenStream) -> Result<TokenStream, Error> {
    let input: syn::ItemFn = syn::parse2(item)?;
    let Handler { ret_level, role, attribute, headers, need_auth } = deluxe::parse2(attr)?;

    let ident = &input.sig.ident;
    if ident != "handler" {
        return Err(syn::Error::new(
            ident.span(),
            "Function derived with `handler` should be named `handler`",
        ));
    }

    let mut skip_debugs: Vec<&syn::Ident> = vec![];
    let mut common_args: Vec<syn::FnArg> = vec![];
    let mut pass_args: Vec<syn::Expr> = vec![];

    for fn_arg in &input.sig.inputs {
        if let syn::FnArg::Typed(arg) = fn_arg
            && let syn::Pat::Ident(pat) = arg.pat.as_ref()
            && pat.ident != "request"
        {
            match pat.ident.to_string().as_str() {
                "database" | "_database" => {
                    skip_debugs.push(&pat.ident);
                    common_args.push(parse_quote! {
                        extract::State(database): extract::State<crate::database::Database>
                    });
                    pass_args.push(parse_quote!(&database));
                }
                "user_id" => {
                    pass_args.push(parse_quote!(user.id));
                }
                _ => match arg.ty.as_ref() {
                    syn::Type::Path(ty) => {
                        if headers.contains(&pat.ident) {
                            if let Some(segment) = ty.path.segments.last()
                                && segment.ident == "Option"
                                && let syn::PathArguments::AngleBracketed(angle) =
                                    &segment.arguments
                                && let Some(syn::GenericArgument::Type(syn::Type::Path(ty))) =
                                    angle.args.first()
                            {
                                common_args
                                    .push(parse_quote!(#pat: Option<axum_extra::TypedHeader<#ty>>));
                                pass_args.push(parse_quote!(#pat.map(|header| header.0)));
                            }
                        } else {
                            if pat.ident == "config" || pat.ident == "informant" {
                                skip_debugs.push(&pat.ident);
                            }
                            common_args.push(
                                parse_quote!(extract::Extension(#pat): extract::Extension<#ty>),
                            );
                            pass_args.push(parse_quote!(#pat));
                        }
                    }
                    syn::Type::Reference(ty) => {
                        if pat.ident == "filesystem" || pat.ident == "config" {
                            skip_debugs.push(&pat.ident);
                        }
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
                },
            }
        }
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

    let is_binary_respone = if let syn::ReturnType::Type(_, ty) = &input.sig.output
        && let syn::Type::Path(ty) = ty.as_ref()
        && let Some(segment) = ty.path.segments.last()
        && segment.ident == "Result"
        && let syn::PathArguments::AngleBracketed(angle) = &segment.arguments
        && let Some(syn::GenericArgument::Type(syn::Type::Path(ty))) = angle.args.first()
        && let Some(segment) = ty.path.segments.first()
        && segment.ident == "binary"
        && let Some(segment) = ty.path.segments.last()
        && segment.ident == "Response"
    {
        true
    } else {
        false
    };

    pass_args.push(parse_quote!(user.request));

    let form_handler = if attribute.form() {
        let form_get_args = [
            common_args.as_slice(),
            [parse_quote!(user: FormGetUser<Request, #need_auth>)].as_slice(),
        ]
        .concat();
        let form_post_args =
            [common_args.as_slice(), [parse_quote!(user: FormPostUser<Request>)].as_slice()]
                .concat();

        if is_binary_respone {
            quote! {
                #[axum::debug_handler]
                pub async fn form_get_handler(
                    #( #form_get_args ),*
                ) -> Result<crate::http::binary::Response, Error> {
                    #ident(#( #pass_args ),*).await
                }

                #[axum::debug_handler]
                pub async fn form_post_handler(
                    #( #form_post_args ),*
                ) -> Result<crate::http::binary::Response, Error> {
                    #ident(#( #pass_args ),*).await
                }
            }
        } else {
            quote! {
                #[axum::debug_handler]
                pub async fn form_get_handler(
                    #( #form_get_args ),*
                ) -> Result<
                        axum::Json<SubsonicResponse<<Request as FormEndpoint>::Response>
                    >, Error> {
                    let response = #ident(#( #pass_args ),*).await?;
                    Ok(axum::Json(SubsonicResponse::new(response)))
                }

                #[axum::debug_handler]
                pub async fn form_post_handler(
                    #( #form_post_args ),*
                ) -> Result<
                        axum::Json<SubsonicResponse<<Request as FormEndpoint>::Response>
                    >, Error> {
                    let response = #ident(#( #pass_args ),*).await?;
                    Ok(axum::Json(SubsonicResponse::new(response)))
                }
            }
        }
    } else {
        quote! {}
    };

    let binary_handler = if attribute.binary() {
        let binary_args = [
            common_args.as_slice(),
            [parse_quote!(user: BinaryUser<Request, #need_auth>)].as_slice(),
        ]
        .concat();

        if is_binary_respone {
            quote! {
                #[axum::debug_handler]
                pub async fn binary_handler(
                    #( #binary_args ),*
                ) -> Result<crate::http::binary::Response, Error> {
                    #ident(#( #pass_args ),*).await
                }
            }
        } else {
            quote! {
                #[axum::debug_handler]
                pub async fn binary_handler(
                    #( #binary_args ),*
                ) -> Result<Vec<u8>, Error> {
                    let response = #ident(#( #pass_args ),*).await?;
                    Ok(bitcode::serialize(&response)?)
                }
            }
        }
    } else {
        quote! {}
    };

    let json_handler = if attribute.json() {
        let json_args = [
            common_args.as_slice(),
            [parse_quote!(user: JsonUser<Request, #need_auth>)].as_slice(),
        ]
        .concat();

        if is_binary_respone {
            quote! {
                #[axum::debug_handler]
                pub async fn json_handler(
                    #( #json_args ),*
                ) -> Result<crate::http::binary::Response, Error> {
                    #ident(#( #pass_args ),*).await
                }
            }
        } else {
            quote! {
                #[axum::debug_handler]
                pub async fn json_handler(
                    #( #json_args ),*
                ) -> Result<Vec<u8>, Error> {
                    let response = #ident(#( #pass_args ),*).await?;
                    Ok(bitcode::serialize(&response)?)
                }
            }
        }
    } else {
        quote! {}
    };

    Ok(quote! {
        #[tracing::instrument(skip(#( #skip_debugs ),*), ret(level = #ret_level), err)]
        #input

        use axum::extract;
        use nghe_api::common::{FormEndpoint, SubsonicResponse};

        use crate::auth::{Authorize, BinaryUser, FormGetUser, FormPostUser, JsonUser};

        impl Authorize for Request {
            fn authorize(role: crate::orm::users::Role) -> Result<(), Error> {
                if #authorize_fn {
                    Ok(())
                } else {
                    Err(Error::MissingRole(#missing_role))
                }
            }
        }

        #form_handler
        #binary_handler
        #json_handler
    })
}

pub fn build_router(item: TokenStream) -> Result<TokenStream, Error> {
    let input = deluxe::parse2::<BuildRouter>(item)?;
    let endpoints: Vec<_> = input
        .modules
        .into_iter()
        .map(|module| {
            let ModuleConfig { ident, attribute } = module.try_into()?;
            let module = ident;

            let mut routers = vec![];

            if attribute.form() {
                let form_get_handler = quote! { #module::form_get_handler };
                let form_post_handler = quote! { #module::form_post_handler };

                let request = quote! { <#module::Request as nghe_api::common::FormURL> };
                routers.push(quote! {
                    route(
                        #request::URL_FORM,
                        axum::routing::get(#form_get_handler).post(#form_post_handler)
                    )
                });
                routers.push(quote! {
                    route(
                        #request::URL_FORM_VIEW,
                        axum::routing::get(#form_get_handler).post(#form_post_handler)
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
        pub fn router(#( #router_args ),*) -> axum::Router<crate::database::Database> {
            #router_body
        }
    })
}
