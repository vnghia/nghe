#![allow(clippy::too_many_lines)]

use concat_string::concat_string;
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
    #[deluxe(default = true)]
    need_auth: bool,
}

#[derive(Debug, deluxe::ExtractAttributes)]
#[deluxe(attributes(handler))]
struct HandlerArg {
    #[deluxe(default = false)]
    header: bool,
}

#[derive(Debug, deluxe::ParseMetaItem)]
struct BuildRouter {
    modules: Vec<syn::Meta>,
    #[deluxe(default = false)]
    filesystem: bool,
    #[deluxe(default = vec![])]
    extensions: Vec<syn::Path>,
}

pub fn handler(attr: TokenStream, item: TokenStream) -> Result<TokenStream, Error> {
    let mut handler: syn::ItemFn = syn::parse2(item)?;
    let Handler { ret_level, role, attribute, need_auth } = deluxe::parse2(attr)?;

    let ident = &handler.sig.ident;
    if ident != "handler" {
        return Err(syn::Error::new(
            ident.span(),
            "Function derived with `handler` should be named `handler`",
        ));
    }

    let mut skip_debugs: Vec<syn::Ident> = vec![];
    let mut common_args: Vec<syn::FnArg> = vec![];
    let mut pass_args: Vec<syn::Expr> = vec![];

    for fn_arg in &mut handler.sig.inputs {
        if let syn::FnArg::Typed(arg) = fn_arg {
            let HandlerArg { header } = deluxe::extract_attributes(arg)?;
            if let syn::Pat::Ident(pat) = arg.pat.as_ref()
                && pat.ident != "request"
            {
                match pat.ident.to_string().as_str() {
                    "database" | "_database" => {
                        skip_debugs.push(pat.ident.clone());
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
                            if header {
                                if let Some(segment) = ty.path.segments.last()
                                    && segment.ident == "Option"
                                    && let syn::PathArguments::AngleBracketed(angle) =
                                        &segment.arguments
                                    && let Some(syn::GenericArgument::Type(syn::Type::Path(ty))) =
                                        angle.args.first()
                                {
                                    common_args.push(
                                        parse_quote!(#pat: Option<axum_extra::TypedHeader<#ty>>),
                                    );
                                    pass_args.push(parse_quote!(#pat.map(|header| header.0)));
                                }
                            } else {
                                if pat.ident == "config" || pat.ident == "informant" {
                                    skip_debugs.push(pat.ident.clone());
                                }
                                common_args.push(
                                    parse_quote!(extract::Extension(#pat): extract::Extension<#ty>),
                                );
                                pass_args.push(parse_quote!(#pat));
                            }
                        }
                        syn::Type::Reference(ty) => {
                            if pat.ident == "filesystem" || pat.ident == "config" {
                                skip_debugs.push(pat.ident.clone());
                            }
                            let ty = ty.elem.as_ref();
                            common_args.push(
                                parse_quote!(extract::Extension(#pat): extract::Extension<#ty>),
                            );
                            pass_args.push(parse_quote!(&#pat));
                        }
                        _ => {
                            return Err(syn::Error::new(
                                arg.ty.span(),
                                "Only path type and reference type are supported for handler \
                                 function",
                            ));
                        }
                    },
                }
            }
        }
    }

    let traced_handler_ident = format_ident!("traced_handler");
    let traced_handler = {
        let source_path = proc_macro::Span::call_site().source_file().path();
        // TODO: Remove this after https://github.com/rust-lang/rust-analyzer/issues/15950.
        let tracing_name = if source_path.as_os_str().is_empty() {
            "handler".to_owned()
        } else {
            let source_dir = source_path.parent().unwrap().file_name().unwrap().to_str().unwrap();
            let source_stem = source_path.file_stem().unwrap().to_str().unwrap();
            concat_string!(source_dir, "::", source_stem)
        };

        let traced_handler_inputs = handler.sig.inputs.clone();
        let traced_handler_args: Vec<_> = traced_handler_inputs
            .iter()
            .map(|arg| {
                if let syn::FnArg::Typed(arg) = arg
                    && let syn::Pat::Ident(pat) = arg.pat.as_ref()
                {
                    Ok(&pat.ident)
                } else {
                    Err(Error::new(arg.span(), "`handler` should only has typed function argument"))
                }
            })
            .try_collect()?;
        let traced_handler_output = &handler.sig.output;

        quote! {
            #[tracing::instrument(
                name = #tracing_name,
                skip(#( #skip_debugs ),*),
                ret(level = #ret_level),
                err
            )]
            #[inline(always)]
            async fn #traced_handler_ident(#traced_handler_inputs) #traced_handler_output {
                #ident(#( #traced_handler_args ),*).await
            }
        }
    };

    let (authorize_fn, missing_role) = if let Some(role) = role {
        if role == "admin" {
            (quote! { role.admin }, role.to_string())
        } else {
            (quote! { role.admin || role.#role }, role.to_string())
        }
    } else {
        (quote! { true }, String::default())
    };

    let is_binary_respone = if let syn::ReturnType::Type(_, ty) = &handler.sig.output
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
                    #traced_handler_ident(#( #pass_args ),*).await
                }

                #[axum::debug_handler]
                pub async fn form_post_handler(
                    #( #form_post_args ),*
                ) -> Result<crate::http::binary::Response, Error> {
                    #traced_handler_ident(#( #pass_args ),*).await
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
                    let response = #traced_handler_ident(#( #pass_args ),*).await?;
                    Ok(axum::Json(SubsonicResponse::new(response)))
                }

                #[axum::debug_handler]
                pub async fn form_post_handler(
                    #( #form_post_args ),*
                ) -> Result<
                        axum::Json<SubsonicResponse<<Request as FormEndpoint>::Response>
                    >, Error> {
                    let response = #traced_handler_ident(#( #pass_args ),*).await?;
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
                    #traced_handler_ident(#( #pass_args ),*).await
                }
            }
        } else {
            quote! {
                #[axum::debug_handler]
                pub async fn binary_handler(
                    #( #binary_args ),*
                ) -> Result<Vec<u8>, Error> {
                    let response = #traced_handler_ident(#( #pass_args ),*).await?;
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
                    #traced_handler_ident(#( #pass_args ),*).await
                }
            }
        } else {
            quote! {
                #[axum::debug_handler]
                pub async fn json_handler(
                    #( #json_args ),*
                ) -> Result<axum::Json<<Request as JsonEndpoint>::Response>, Error> {
                    let response = #traced_handler_ident(#( #pass_args ),*).await?;
                    Ok(axum::Json(response))
                }
            }
        }
    } else {
        quote! {}
    };

    Ok(quote! {
        #handler

        #traced_handler

        use axum::extract;
        use nghe_api::common::{FormEndpoint, JsonEndpoint, SubsonicResponse};

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
