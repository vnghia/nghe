use concat_string::concat_string;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_str, Error, ExprPath, ItemStruct};

use crate::{get_base_name, get_caller_types_module};

const TYPES_CONSTANT_RESPONSE_IMPORT_PREFIX: &str = "crate::response";
const BACKEND_COMMON_ERROR_IMPORT_PREFIX: &str = "crate::open_subsonic::common::error";

#[derive(deluxe::ParseMetaItem)]
struct AddSubsonicResponse {
    #[deluxe(default = true)]
    success: bool,
}

pub fn add_subsonic_response(args: TokenStream, input: TokenStream) -> Result<TokenStream, Error> {
    let item_struct: ItemStruct = syn::parse2(input)?;
    let item_ident = &item_struct.ident;
    let base_type = get_base_name(item_ident, "Body")?;

    let args = deluxe::parse2::<AddSubsonicResponse>(args)?;
    let constant_type = parse_str::<ExprPath>(&if args.success {
        concat_string!(TYPES_CONSTANT_RESPONSE_IMPORT_PREFIX, "::SuccessConstantResponse")
    } else {
        concat_string!(TYPES_CONSTANT_RESPONSE_IMPORT_PREFIX, "::ErrorConstantResponse")
    })?;
    let root_type = parse_str::<ExprPath>(&concat_string!(
        TYPES_CONSTANT_RESPONSE_IMPORT_PREFIX,
        "::RootResponse"
    ))?;
    let subsonic_type = parse_str::<ExprPath>(&concat_string!(
        TYPES_CONSTANT_RESPONSE_IMPORT_PREFIX,
        "::SubsonicResponse"
    ))?;

    let root_ident = format_ident!("{}RootResponse", base_type);
    let subsonic_ident = format_ident!("{}SubsonicResponse", base_type);

    Ok(quote! {
        #[nghe_proc_macros::add_types_derive]
        #item_struct

        pub type #root_ident = #root_type<#constant_type, #item_ident>;

        pub type #subsonic_ident = #subsonic_type<#constant_type, #item_ident>;
    })
}

pub fn add_axum_response(item_ident: TokenStream) -> Result<TokenStream, Error> {
    let types_path = get_caller_types_module();

    let item_ident = format_ident!("{}", item_ident.to_string());
    let base_type = get_base_name(&item_ident, "Body")?;

    let json_response_path = parse_str::<ExprPath>(&concat_string!(
        BACKEND_COMMON_ERROR_IMPORT_PREFIX,
        "::ServerJsonResponse"
    ))?;

    let json_response_ident = format_ident!("{}JsonResponse", base_type);
    let subsonic_ident = format_ident!("{}SubsonicResponse", base_type);

    Ok(quote! {
        use #types_path::*;

        pub type #json_response_ident = #json_response_path<#subsonic_ident>;
    })
}
