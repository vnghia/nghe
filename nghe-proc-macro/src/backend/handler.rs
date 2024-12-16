use concat_string::concat_string;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{parse_quote, Error};

use crate::endpoint::Attribute;

#[derive(Debug, deluxe::ParseMetaItem)]
struct Config {
    role: Option<syn::Ident>,
    #[deluxe(flatten)]
    attribute: Attribute,
    #[deluxe(default = true)]
    need_auth: bool,
}

#[derive(Debug, deluxe::ExtractAttributes)]
#[deluxe(attributes(handler))]
struct ArgConfig {
    #[deluxe(default = false)]
    header: bool,
}

#[derive(Debug)]
enum Arg {
    Database { ident: syn::Ident, use_database: bool },
    User(syn::Ident),
    Request,
    Extension { ident: syn::Ident, ty: syn::TypePath, reference: bool },
    Header { ident: syn::Ident, ty: syn::TypePath },
}

#[derive(Debug)]
struct Args {
    value: Vec<Arg>,
    use_user: bool,
    use_request: bool,
}

#[derive(Debug)]
pub struct Handler {
    item: syn::ItemFn,
    config: Config,
    args: Args,
    // None if not `Result` and false if not `Result<binary::Response>`.
    is_result_binary: Option<bool>,
}

impl Arg {
    fn new(arg: &mut syn::FnArg) -> Result<Self, Error> {
        if let syn::FnArg::Typed(arg) = arg
            && let config = deluxe::extract_attributes::<_, ArgConfig>(arg)?
            && let syn::Pat::Ident(pat) = arg.pat.as_ref()
        {
            match pat.ident.to_string().as_str() {
                "database" => Ok(Self::Database { ident: pat.ident.clone(), use_database: true }),
                "user_id" => Ok(Self::User(parse_quote!(id))),
                "request" => Ok(Self::Request),
                _ => {
                    let ty = if config.header {
                        if let syn::Type::Path(ty) = arg.ty.as_ref()
                            && let Some(segment) = ty.path.segments.last()
                            && segment.ident == "Option"
                            && let syn::PathArguments::AngleBracketed(angle) = &segment.arguments
                            && let Some(syn::GenericArgument::Type(ty)) = angle.args.first()
                        {
                            ty
                        } else {
                            return Err(syn::Error::new(
                                arg.ty.span(),
                                "Header type must be wrapped with an `Option`",
                            ));
                        }
                    } else {
                        arg.ty.as_ref()
                    };

                    let (ty, reference) = match ty {
                        syn::Type::Path(ty) => (ty.clone(), false),
                        syn::Type::Reference(ty) if let syn::Type::Path(ty) = ty.elem.as_ref() => {
                            (ty.clone(), true)
                        }
                        _ => {
                            return Err(syn::Error::new(
                                arg.ty.span(),
                                "Only path type or reference of path type are supported for \
                                 handler function",
                            ));
                        }
                    };

                    if config.header {
                        Ok(Self::Header { ident: pat.ident.clone(), ty })
                    } else {
                        Ok(Self::Extension { ident: pat.ident.clone(), ty, reference })
                    }
                }
            }
        } else {
            Err(syn::Error::new(
                arg.span(),
                "Function derived with `handler` should have typed ident input only",
            ))
        }
    }

    fn to_skip_debug(&self) -> Option<&syn::Ident> {
        match self {
            Arg::Database { ident, use_database } => {
                if *use_database {
                    Some(ident)
                } else {
                    None
                }
            }
            Arg::Extension { ident, .. } => Some(ident),
            _ => None,
        }
    }

    fn to_arg_expr(&self) -> (Option<syn::FnArg>, Option<syn::Expr>) {
        match self {
            Arg::Database { ident, use_database } => (
                Some(parse_quote! {
                    axum::extract::State(#ident): axum::extract::State<crate::database::Database>
                }),
                if *use_database { Some(parse_quote!(&#ident)) } else { None },
            ),
            Arg::User(ident) => (None, Some(parse_quote!(user.#ident))),
            Arg::Request => (None, None),
            Arg::Extension { ident, ty, reference, .. } => (
                Some(
                    parse_quote! {axum::extract::Extension(#ident): axum::extract::Extension<#ty>},
                ),
                Some(if *reference { parse_quote!(&#ident) } else { parse_quote!(#ident) }),
            ),
            Arg::Header { ident, ty } => (
                Some(parse_quote! {
                    #ident: Option<axum_extra::TypedHeader<#ty>>
                }),
                Some(parse_quote!(#ident.map(|header| header.0))),
            ),
        }
    }
}

impl Args {
    fn new(args: &mut Punctuated<syn::FnArg, syn::Token![,]>) -> Result<Self, Error> {
        let mut value = args.iter_mut().map(Arg::new).try_collect::<Vec<_>>()?;
        if !value.iter().any(|arg| matches!(arg, Arg::Database { .. })) {
            // Need for authentication or setup.
            value.push(Arg::Database { ident: format_ident!("_database"), use_database: false });
        }
        let use_user = value.iter().any(|arg| matches!(arg, Arg::User(_)));
        let use_request = value.iter().any(|arg| matches!(arg, Arg::Request));
        Ok(Self { value, use_user, use_request })
    }
}

impl Handler {
    pub fn new(attr: TokenStream, item: TokenStream) -> Result<Self, Error> {
        let mut item: syn::ItemFn = syn::parse2(item)?;
        let ident = &item.sig.ident;
        if ident != "handler" {
            return Err(syn::Error::new(
                ident.span(),
                "Function derived with `handler` should be named `handler`",
            ));
        }

        let config = deluxe::parse2(attr)?;
        let args = Args::new(&mut item.sig.inputs)?;
        let is_result_binary = Self::is_result_binary(&item.sig.output);

        Ok(Self { item, config, args, is_result_binary })
    }

    pub fn build(&self) -> Result<TokenStream, Error> {
        let traced_handler = self.traced_handler()?;
        let authz = self.authz();

        let user_ident = if self.args.use_user || self.args.use_request {
            format_ident!("user")
        } else {
            format_ident!("_user")
        };

        let form_handler = if self.config.attribute.form() {
            let mut additional_args = vec![];
            let mut additional_exprs = vec![];

            if self.config.need_auth {
                additional_args
                    .push(parse_quote!(#user_ident: crate::http::extract::auth::Form<Request>));
            }
            if self.args.use_request {
                additional_exprs.push(parse_quote!(user.request));
            }

            Some(self.handler(
                "form",
                &traced_handler.sig.ident,
                additional_args,
                additional_exprs,
                &parse_quote! {
                    Result<
                        axum::Json<
                            nghe_api::common::SubsonicResponse<
                                <Request as nghe_api::common::FormEndpoint>::Response
                            >
                        >,
                        crate::Error
                    >
                },
                &parse_quote! {
                    axum::Json(nghe_api::common::SubsonicResponse::new(response))
                },
            ))
        } else {
            None
        };

        let binary_handler = if self.config.attribute.binary() {
            let mut additional_args = vec![];
            let mut additional_exprs = vec![];

            if self.config.need_auth {
                additional_args
                    .push(parse_quote!(#user_ident: crate::http::extract::auth::Header<Request>));
            }
            if self.args.use_request {
                additional_args.push(parse_quote! {
                    crate::http::extract::Binary(request): crate::http::extract::Binary<Request>
                });
                additional_exprs.push(parse_quote!(request));
            }

            Some(self.handler(
                "binary",
                &traced_handler.sig.ident,
                additional_args,
                additional_exprs,
                &parse_quote!(Result<Vec<u8>, crate::Error>),
                &parse_quote!(bitcode::serialize(&response)?),
            ))
        } else {
            None
        };

        let json_handler = if self.config.attribute.json() {
            let mut additional_args = vec![];
            let mut additional_exprs = vec![];

            if self.config.need_auth {
                additional_args
                    .push(parse_quote!(#user_ident: crate::http::extract::auth::Header<Request>));
            }
            if self.args.use_request {
                additional_args.push(parse_quote!(axum::Json(request): axum::Json<Request>));
                additional_exprs.push(parse_quote!(request));
            }

            Some(self.handler(
                "json",
                &traced_handler.sig.ident,
                additional_args,
                additional_exprs,
                &parse_quote! {
                    Result<
                        axum::Json<
                            <Request as nghe_api::common::JsonEndpoint>::Response
                        >,
                        crate::Error
                    >
                },
                &parse_quote!(axum::Json(response)),
            ))
        } else {
            None
        };

        let handler = &self.item;
        Ok(quote! {
            #handler

            #traced_handler

            #authz

            #form_handler
            #binary_handler
            #json_handler
        })
    }

    fn is_result_binary(output: &syn::ReturnType) -> Option<bool> {
        if let syn::ReturnType::Type(_, ty) = &output
            && let syn::Type::Path(ty) = ty.as_ref()
            && let Some(segment) = ty.path.segments.last()
            && segment.ident == "Result"
        {
            Some(
                if let syn::PathArguments::AngleBracketed(angle) = &segment.arguments
                    && let Some(syn::GenericArgument::Type(syn::Type::Path(ty))) =
                        angle.args.first()
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
            None
        }
    }

    fn traced_handler(&self) -> Result<syn::ItemFn, Error> {
        let source_path = proc_macro::Span::call_site().source_file().path();
        // TODO: Remove this after https://github.com/rust-lang/rust-analyzer/issues/15950.
        let tracing_name = if source_path.as_os_str().is_empty() {
            "handler".to_owned()
        } else {
            let source_dir = source_path.parent().unwrap().file_name().unwrap().to_str().unwrap();
            let source_stem = source_path.file_stem().unwrap().to_str().unwrap();
            concat_string!(source_dir, ":", source_stem)
        };

        let mut sig = self.item.sig.clone();
        sig.ident = format_ident!("traced_handler");
        let args: Punctuated<&syn::Ident, syn::Token![,]> = sig
            .inputs
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

        let skip_debugs: Punctuated<&syn::Ident, syn::Token![,]> =
            self.args.value.iter().filter_map(Arg::to_skip_debug).collect();
        let mut tracing_args = Punctuated::<syn::Meta, syn::Token![,]>::default();
        tracing_args.push(parse_quote!(name = #tracing_name));
        tracing_args.push(parse_quote!(skip(#skip_debugs)));
        if self.is_result_binary.is_some() {
            tracing_args.push(parse_quote!(ret(level = "trace")));
            tracing_args.push(parse_quote!(err(Debug)));
        }

        let handler_ident = &self.item.sig.ident;
        let source: syn::Expr = if sig.asyncness.is_some() {
            parse_quote!(#handler_ident(#args).await)
        } else {
            parse_quote!(#handler_ident(#args))
        };

        Ok(parse_quote! {
            #[coverage(off)]
            #[inline(always)]
            #[tracing::instrument(#tracing_args)]
            #sig { #source }
        })
    }

    fn authz(&self) -> syn::ItemImpl {
        let source: syn::Expr = if let Some(role) = self.config.role.as_ref() {
            if role == "admin" {
                parse_quote!(role.admin)
            } else {
                parse_quote!(role.admin || role.#role)
            }
        } else {
            parse_quote!(true)
        };

        parse_quote! {
            #[coverage(off)]
            impl crate::http::extract::auth::AuthZ for Request {
                #[inline(always)]
                fn is_authorized(role: crate::orm::users::Role) ->  bool {
                    #source
                }
            }
        }
    }

    fn handler(
        &self,
        prefix: &'static str,
        traced_handler: &syn::Ident,
        additional_args: Vec<syn::FnArg>,
        additional_exprs: Vec<syn::Expr>,
        result: &syn::Type,
        response: &syn::Expr,
    ) -> syn::ItemFn {
        let ident = format_ident!("{prefix}_handler");
        let (args, exprs): (Vec<_>, Vec<_>) =
            self.args.value.iter().map(Arg::to_arg_expr).collect();
        let args: Punctuated<syn::FnArg, syn::Token![,]> =
            args.into_iter().flatten().chain(additional_args).collect();
        let exprs: Punctuated<syn::Expr, syn::Token![,]> =
            exprs.into_iter().flatten().chain(additional_exprs).collect();

        let asyncness = self.item.sig.asyncness.map(|_| quote!(.await));
        let tryness = self.is_result_binary.map(|_| quote!(?));

        if self.is_result_binary.is_some_and(std::convert::identity) {
            parse_quote! {
                #[coverage(off)]
                #[axum::debug_handler]
                pub async fn #ident(#args) -> Result<crate::http::binary::Response, crate::Error> {
                    #traced_handler(#exprs)#asyncness
                }
            }
        } else {
            parse_quote! {
                #[coverage(off)]
                #[axum::debug_handler]
                pub async fn #ident(#args) -> #result {
                    let response = #traced_handler(#exprs)
                        #asyncness
                        #tryness;
                    Ok(#response)
                }
            }
        }
    }
}