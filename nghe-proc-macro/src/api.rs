use concat_string::concat_string;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_quote, parse_str, Error};

#[derive(Debug, deluxe::ExtractAttributes)]
#[deluxe(attributes(endpoint))]
struct Endpoint {
    path: String,
    #[deluxe(default = true)]
    json: bool,
    #[deluxe(default = true)]
    binary: bool,
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
    let Endpoint { path, json, binary, url_only, same_crate } =
        deluxe::extract_attributes(&mut input)?;

    let ident = &input.ident;
    if ident != "Request" {
        return Err(syn::Error::new(
            ident.span(),
            "Struct derived with `Endpoint` should be named `Request`",
        ));
    }

    let crate_path = if same_crate { format_ident!("crate") } else { format_ident!("nghe_api") };

    let impl_json = if json {
        let url = concat_string!("/rest/", &path);
        let url_view = concat_string!("/rest/", &path, ".view");

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
                const URL: &'static str = #url;
                const URL_VIEW: &'static str = #url_view;
            }

            #impl_endpoint
        }
    } else {
        quote! {}
    };

    let impl_binary = if binary {
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

    Ok(quote! {
        #impl_json
        #impl_binary
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
        #[derive(#(#derives),*)]
        #( #attributes )*
        #input
    })
}
