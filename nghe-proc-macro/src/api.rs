use concat_string::concat_string;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_str, Error, Ident};

#[derive(Debug, deluxe::ExtractAttributes)]
#[deluxe(attributes(endpoint))]
struct Endpoint {
    path: String,
    #[deluxe(default = format_ident!("Response"))]
    response: Ident,
    #[deluxe(default = true)]
    same_crate: bool,
}

#[derive(Debug, deluxe::ParseMetaItem)]
struct Derive {
    #[deluxe(default = true)]
    debug: bool,
    #[deluxe(default = true)]
    serde: bool,
    #[deluxe(default = true)]
    binary: bool,
    #[deluxe(default = false)]
    endpoint: bool,
    #[deluxe(default = true)]
    response: bool,
    #[deluxe(default = true)]
    fake: bool,
}

pub fn derive_endpoint(item: TokenStream) -> Result<TokenStream, Error> {
    let mut input: syn::DeriveInput = syn::parse2(item)?;
    let Endpoint { path, response, same_crate } = deluxe::extract_attributes(&mut input)?;

    let ident = &input.ident;
    if ident != "Request" {
        return Err(syn::Error::new(
            ident.span(),
            "Struct derived with `Endpoint` should be named `Request`",
        ));
    }

    let endpoint = concat_string!("/rest/", &path);
    let endpoint_view = concat_string!("/rest/", &path, ".view");
    let endpoint_binary = concat_string!("/rest/", &path, ".bin");

    let crate_path = if same_crate { format_ident!("crate") } else { format_ident!("nghe_api") };

    Ok(quote! {
        impl #crate_path::common::Endpoint for #ident {
            const ENDPOINT: &'static str = #endpoint;
            const ENDPOINT_VIEW: &'static str = #endpoint_view;
            const ENDPOINT_BINARY: &'static str = #endpoint_binary;

            type Response = #response;
        }
    })
}

pub fn derive(args: TokenStream, item: TokenStream) -> Result<TokenStream, Error> {
    let args: Derive = deluxe::parse2(args)?;
    let input: syn::DeriveInput = syn::parse2(item)?;

    let is_request = args.endpoint || !args.response;

    let mut derives: Vec<syn::Expr> = vec![];

    if args.serde {
        if is_request {
            derives.push(parse_str("serde::Deserialize")?);
        } else {
            derives.push(parse_str("serde::Serialize")?);
        }
    }

    if args.debug {
        derives.push(parse_str("Debug")?);
    }
    if args.binary {
        derives.push(parse_str("bitcode::Encode")?);
        derives.push(parse_str("bitcode::Decode")?);
    }
    if args.endpoint {
        derives.push(parse_str("nghe_proc_macro::Endpoint")?);
    }

    let serde_rename = if args.serde {
        quote! { #[serde(rename_all = "camelCase")] }
    } else {
        quote! {}
    };

    let fake_derive = if is_request && args.fake {
        quote! { #[cfg_attr(feature = "fake", derive(fake::Dummy))] }
    } else {
        quote! {}
    };

    Ok(quote! {
        #[derive(#(#derives),*)]
        #serde_rename
        #fake_derive
        #input
    })
}
