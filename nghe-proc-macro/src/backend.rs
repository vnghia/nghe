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

    let mut use_database = false;
    let mut use_user_id = false;
    let mut use_request = false;

    for fn_arg in &mut handler.sig.inputs {
        if let syn::FnArg::Typed(arg) = fn_arg {
            let HandlerArg { header } = deluxe::extract_attributes(arg)?;
            if let syn::Pat::Ident(pat) = arg.pat.as_ref() {
                match pat.ident.to_string().as_str() {
                    "database" => {
                        use_database = true;
                        skip_debugs.push(pat.ident.clone());
                        common_args.push(parse_quote! {
                            extract::State(database): extract::State<crate::database::Database>
                        });
                        pass_args.push(parse_quote!(&database));
                    }
                    "user_id" => {
                        use_user_id = true;
                        pass_args.push(parse_quote!(user.id));
                    }
                    "request" => {
                        use_request = true;
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

    let (is_return_result, is_binary_response) = if let syn::ReturnType::Type(_, ty) =
        &handler.sig.output
        && let syn::Type::Path(ty) = ty.as_ref()
        && let Some(segment) = ty.path.segments.last()
        && segment.ident == "Result"
    {
        (
            true,
            if let syn::PathArguments::AngleBracketed(angle) = &segment.arguments
                && let Some(syn::GenericArgument::Type(syn::Type::Path(ty))) = angle.args.first()
                && let Some(segment) = ty.path.segments.first()
                && segment.ident == "binary"
                && let Some(segment) = ty.path.segments.last()
                && segment.ident == "Response"
            {
                true
            } else {
                false
            },
        )
    } else {
        (false, false)
    };

    let traced_handler_ident = format_ident!("traced_handler");
    let (traced_handler, traced_handler_await) = {
        let source_path = proc_macro::Span::call_site().source_file().path();
        // TODO: Remove this after https://github.com/rust-lang/rust-analyzer/issues/15950.
        let tracing_name = if source_path.as_os_str().is_empty() {
            "handler".to_owned()
        } else {
            let source_dir = source_path.parent().unwrap().file_name().unwrap().to_str().unwrap();
            let source_stem = source_path.file_stem().unwrap().to_str().unwrap();
            concat_string!(source_dir, "::", source_stem)
        };

        let handler_async = handler.sig.asyncness;
        let handler_await = if handler_async.is_some() { Some(quote! {.await}) } else { None };

        let handler_inputs = handler.sig.inputs.clone();
        let handler_args: Vec<_> = handler_inputs
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
        let handler_output = &handler.sig.output;

        let mut tracing_args =
            vec![quote! {name = #tracing_name}, quote! {skip(#( #skip_debugs ),*)}];
        if is_return_result {
            tracing_args.push(quote! {ret(level = #ret_level)});
            tracing_args.push(quote! {err(Debug)});
        }

        (
            quote! {
                #[tracing::instrument(#( #tracing_args ),*)]
                #[inline(always)]
                #[coverage(off)]
                #handler_async fn #traced_handler_ident(#handler_inputs) #handler_output {
                    #ident(#( #handler_args ),*)#handler_await
                }
            },
            handler_await,
        )
    };
    let traced_handler_try = if is_return_result { Some(quote! {?}) } else { None };

    let is_authorized = if let Some(role) = role {
        if role == "admin" {
            quote! { role.admin }
        } else {
            quote! { role.admin || role.#role }
        }
    } else {
        quote! { true }
    };

    if !use_database {
        common_args.push(parse_quote! {
            extract::State(_database): extract::State<crate::database::Database>
        });
    }

    let user_ident =
        if use_user_id || use_request { format_ident!("user") } else { format_ident!("_user") };

    let form_handler = if attribute.form() {
        let mut form_args = common_args.clone();
        let mut pass_args = pass_args.clone();

        if need_auth || use_request {
            form_args.push(parse_quote!(#user_ident: crate::http::extract::auth::Form<Request>));
        }
        if use_request {
            pass_args.push(parse_quote!(user.request));
        }

        if is_binary_response {
            quote! {
                #[axum::debug_handler]
                #[coverage(off)]
                pub async fn form_handler(
                    #( #form_args ),*
                ) -> Result<crate::http::binary::Response, crate::Error> {
                    #traced_handler_ident(#( #pass_args ),*)#traced_handler_await
                }
            }
        } else {
            quote! {
                #[axum::debug_handler]
                #[coverage(off)]
                pub async fn form_handler(
                    #( #form_args ),*
                ) -> Result<
                        axum::Json<SubsonicResponse<<Request as FormEndpoint>::Response>
                    >, crate::Error> {
                    let response = #traced_handler_ident(#( #pass_args ),*)
                        #traced_handler_await
                        #traced_handler_try;
                    Ok(axum::Json(SubsonicResponse::new(response)))
                }
            }
        }
    } else {
        quote! {}
    };

    let binary_handler = if attribute.binary() {
        let mut binary_args = common_args.clone();
        let mut pass_args = pass_args.clone();

        if need_auth {
            binary_args
                .push(parse_quote!(#user_ident: crate::http::extract::auth::Header<Request>));
        }
        if use_request {
            binary_args.push(parse_quote!(
                crate::http::extract::Binary(request): crate::http::extract::Binary<Request>
            ));
            pass_args.push(parse_quote!(request));
        }

        if is_binary_response {
            quote! {
                #[axum::debug_handler]
                #[coverage(off)]
                pub async fn binary_handler(
                    #( #binary_args ),*
                ) -> Result<crate::http::binary::Response, crate::Error> {
                    #traced_handler_ident(#( #pass_args ),*)#traced_handler_await
                }
            }
        } else {
            quote! {
                #[axum::debug_handler]
                #[coverage(off)]
                pub async fn binary_handler(
                    #( #binary_args ),*
                ) -> Result<Vec<u8>, crate::Error> {
                    let response = #traced_handler_ident(#( #pass_args ),*)
                        #traced_handler_await
                        #traced_handler_try;
                    Ok(bitcode::serialize(&response)?)
                }
            }
        }
    } else {
        quote! {}
    };

    let json_handler = if attribute.json() {
        let mut json_args = common_args.clone();
        let mut pass_args = pass_args.clone();

        if need_auth {
            json_args.push(parse_quote!(#user_ident: crate::http::extract::auth::Header<Request>));
        }
        if use_request {
            json_args.push(parse_quote!(axum::Json(request): axum::Json<Request>));
            pass_args.push(parse_quote!(request));
        }

        if is_binary_response {
            quote! {
                #[axum::debug_handler]
                #[coverage(off)]
                pub async fn json_handler(
                    #( #json_args ),*
                ) -> Result<crate::http::binary::Response, crate::Error> {
                    #traced_handler_ident(#( #pass_args ),*)#traced_handler_await
                }
            }
        } else {
            quote! {
                #[axum::debug_handler]
                #[coverage(off)]
                pub async fn json_handler(
                    #( #json_args ),*
                ) -> Result<axum::Json<<Request as JsonEndpoint>::Response>, crate::Error> {
                    let response = #traced_handler_ident(#( #pass_args ),*)
                        #traced_handler_await
                        #traced_handler_try;
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

        #[coverage(off)]
        impl crate::http::extract::auth::AuthZ for Request {
            #[inline(always)]
            fn is_authorized(role: crate::orm::users::Role) ->  bool {
                #is_authorized
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
        pub fn router(#( #router_args ),*) -> axum::Router<crate::database::Database> {
            #router_body
        }
    })
}
