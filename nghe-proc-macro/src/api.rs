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
}

#[derive(Debug, deluxe::ParseMetaItem)]
struct Derive {
    #[deluxe(default = true)]
    debug: bool,
    #[deluxe(default = true)]
    bitcode: bool,
    #[deluxe(default = false)]
    endpoint: bool,
    #[deluxe(default = true)]
    response: bool,
}

pub fn derive_endpoint(item: TokenStream) -> Result<TokenStream, Error> {
    let mut input: syn::DeriveInput = syn::parse2(item)?;
    let Endpoint { path, response } = deluxe::extract_attributes(&mut input)?;

    let ident = &input.ident;
    if ident != "Request" {
        return Err(syn::Error::new(
            ident.span(),
            "Struct derived with `Endpoint` should be named `Request`",
        ));
    }

    let endpoint = concat_string!("/rest/", &path);
    let endpoint_view = concat_string!("/rest/", &path, ".view");

    Ok(quote! {
        impl crate::common::Endpoint for #ident {
            const ENDPOINT: &'static str = #endpoint;
            const ENDPOINT_VIEW: &'static str = #endpoint_view;

            type Response = #response;
        }
    })
}

pub fn derive(args: TokenStream, item: TokenStream) -> Result<TokenStream, Error> {
    let args: Derive = deluxe::parse2(args)?;
    let input: syn::ItemStruct = syn::parse2(item)?;

    let is_request = args.endpoint || !args.response;

    let mut derives: Vec<syn::Expr> = vec![];

    if is_request {
        derives.push(parse_str("serde::Deserialize")?);
    } else {
        derives.push(parse_str("serde::Serialize")?);
    }
    if args.debug {
        derives.push(parse_str("Debug")?);
    }
    if args.bitcode {
        derives.push(parse_str("bitcode::Encode")?);
        derives.push(parse_str("bitcode::Decode")?);
    }
    if args.endpoint {
        derives.push(parse_str("nghe_proc_macro::Endpoint")?);
    }

    Ok(quote! {
        #[derive(#(#derives),*)]
        #[serde(rename_all = "camelCase")]
        #input
    })
}
