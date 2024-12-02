use concat_string::concat_string;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_quote, parse_str, Error};

use crate::endpoint::Attribute;

#[derive(Debug, deluxe::ExtractAttributes)]
#[deluxe(attributes(endpoint))]
struct Endpoint {
    path: String,
    #[deluxe(flatten)]
    attribute: Attribute,
    #[deluxe(default = false)]
    url_only: bool,
    #[deluxe(default = true)]
    same_crate: bool,
}

#[derive(Debug, deluxe::ParseMetaItem)]
struct Derive {
    #[deluxe(default = true)]
    request: bool,
    #[deluxe(default = true)]
    response: bool,
    #[deluxe(default = false)]
    endpoint: bool,
    #[deluxe(default = true)]
    debug: bool,
    #[deluxe(default = true)]
    serde_apply: bool,
    #[deluxe(default = false)]
    serde_as: bool,
    #[deluxe(default = false)]
    fake: bool,
    #[deluxe(default = true)]
    copy: bool,
    #[deluxe(default = true)]
    eq: bool,
    #[deluxe(default = true)]
    ord: bool,
    #[deluxe(default = true)]
    test_only: bool,
}

pub fn derive_endpoint(item: TokenStream) -> Result<TokenStream, Error> {
    let mut input: syn::DeriveInput = syn::parse2(item)?;
    let Endpoint { path, attribute, url_only, same_crate } =
        deluxe::extract_attributes(&mut input)?;

    let ident = &input.ident;
    if ident != "Request" {
        return Err(syn::Error::new(
            ident.span(),
            "Struct derived with `Endpoint` should be named `Request`",
        ));
    }

    let crate_path = if same_crate { format_ident!("crate") } else { format_ident!("nghe_api") };

    let impl_form = if attribute.form() {
        let url_form = concat_string!("/rest/", &path);
        let url_form_view = concat_string!("/rest/", &path, ".view");

        let impl_endpoint = if url_only {
            quote! {}
        } else {
            quote! {
                impl #crate_path::common::FormEndpoint for #ident {
                    type Response = Response;
                }
            }
        };

        quote! {
            impl #crate_path::common::FormURL for #ident {
                const URL_FORM: &'static str = #url_form;
                const URL_FORM_VIEW: &'static str = #url_form_view;
            }

            #impl_endpoint
        }
    } else {
        quote! {}
    };

    let impl_binary = if attribute.binary() {
        let url_binary = concat_string!("/rest/", &path, ".bin");

        let impl_endpoint = if url_only {
            quote! {}
        } else {
            quote! {
                impl #crate_path::common::BinaryEndpoint for #ident {
                    type Response = Response;
                }
            }
        };

        quote! {
            impl #crate_path::common::BinaryURL for #ident {
                const URL_BINARY: &'static str = #url_binary;
            }

            #impl_endpoint
        }
    } else {
        quote! {}
    };

    let impl_json = if attribute.json() {
        let url_json = concat_string!("/rest/", &path, ".json");

        let impl_endpoint = if url_only {
            quote! {}
        } else {
            quote! {
                impl #crate_path::common::JsonEndpoint for #ident {
                    type Response = Response;
                }
            }
        };

        quote! {
            impl #crate_path::common::JsonURL for #ident {
                const URL_JSON: &'static str = #url_json;
            }

            #impl_endpoint
        }
    } else {
        quote! {}
    };

    Ok(quote! {
        #impl_form
        #impl_binary
        #impl_json
    })
}

pub fn derive(args: TokenStream, item: TokenStream) -> Result<TokenStream, Error> {
    let args: Derive = deluxe::parse2(args)?;
    let input: syn::DeriveInput = syn::parse2(item)?;

    let ident = input.ident.to_string();
    let is_request_struct = ident == "Request";
    let is_request = args.request || ident.ends_with("Request");
    let is_response = args.response || ident.ends_with("Response");
    let has_serde = is_request || is_response;

    let is_enum = matches!(input.data, syn::Data::Enum(_));

    let mut derives: Vec<syn::Expr> = vec![];
    let mut attributes: Vec<syn::Attribute> = vec![];

    if is_request {
        derives.push(parse_str("::serde::Deserialize")?);
    }
    if is_response {
        derives.push(parse_str("::serde::Serialize")?);
    }

    if args.debug {
        derives.push(parse_str("Debug")?);
    }

    if args.endpoint || is_request_struct {
        derives.push(parse_str("nghe_proc_macro::Endpoint")?);
    }

    if has_serde {
        if is_enum {
            attributes.push(parse_quote!(#[serde(rename_all_fields = "camelCase")]));
        }
        attributes.push(parse_quote!(#[serde(rename_all = "camelCase")]));
    };

    let apply_statement = if has_serde && args.serde_apply {
        quote! {
            #[serde_with::apply(
                Option => #[serde(skip_serializing_if = "Option::is_none", default)],
                Vec => #[serde(skip_serializing_if = "Vec::is_empty", default)],
                date::Date => #[serde(skip_serializing_if = "date::Date::is_none", default)],
                genre::Genres => #[serde(
                    skip_serializing_if = "genre::Genres::is_empty",
                    default
                )],
                OffsetDateTime => #[serde(with = "crate::time::serde")],
                Option<OffsetDateTime> => #[serde(
                    with = "crate::time::serde::option",
                    skip_serializing_if = "Option::is_none",
                    default
                )],
                time::Duration => #[serde(with = "crate::time::duration::serde")],
            )]
        }
    } else {
        quote! {}
    };
    let as_statement = if has_serde && args.serde_as {
        quote! { #[serde_with::serde_as] }
    } else {
        quote! {}
    };

    if args.fake {
        attributes
            .push(parse_quote!(#[cfg_attr(any(test, feature = "fake"), derive(fake::Dummy))]));
    }

    if is_enum {
        derives.extend_from_slice(
            &["Clone", "PartialEq", "Eq", "PartialOrd", "Ord"]
                .into_iter()
                .map(parse_str)
                .try_collect::<Vec<_>>()?,
        );
        if args.copy {
            derives.push(parse_str("Copy")?);
        }
    }

    if !is_enum && args.eq {
        if args.test_only {
            attributes.push(
                parse_quote!(#[cfg_attr(any(test, feature = "test"), derive(PartialEq, Eq))]),
            );
        } else {
            derives.extend_from_slice(
                &["PartialEq", "Eq"].into_iter().map(parse_str).try_collect::<Vec<_>>()?,
            );
        }
    }

    if !is_enum && args.ord {
        if args.test_only {
            attributes.push(
                parse_quote!(#[cfg_attr(any(test, feature = "test"), derive(PartialOrd, Ord))]),
            );
        } else {
            derives.extend_from_slice(
                &["PartialOrd", "Ord"].into_iter().map(parse_str).try_collect::<Vec<_>>()?,
            );
        }
    }

    Ok(quote! {
        #apply_statement
        #as_statement
        #[derive(#(#derives),*)]
        #( #attributes )*
        #input
    })
}
