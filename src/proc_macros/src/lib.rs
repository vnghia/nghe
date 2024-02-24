use concat_string::concat_string;
use darling::{ast::NestedMeta, Error, FromMeta};
use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, Ident, ItemStruct};

const CONSTANT_RESPONSE_IMPORT_PREFIX: &str = "crate::open_subsonic::common::response";
const COMMON_REQUEST_IMPORT_PREFIX: &str = "crate::open_subsonic::common::request";

#[derive(Debug, FromMeta)]
struct WrapSubsonicResponse {
    #[darling(default = "return_true")]
    success: bool,
}

#[proc_macro_attribute]
pub fn wrap_subsonic_response(args: TokenStream, input: TokenStream) -> TokenStream {
    let old_struct = parse_macro_input!(input as ItemStruct);

    let attr_args = match NestedMeta::parse_meta_list(args.into()) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(Error::from(e).write_errors());
        }
    };
    let _args = match WrapSubsonicResponse::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };

    let constant_type = if _args.success {
        concat_string!(CONSTANT_RESPONSE_IMPORT_PREFIX, "::SuccessConstantResponse")
    } else {
        concat_string!(CONSTANT_RESPONSE_IMPORT_PREFIX, "::ErrorConstantResponse")
    };
    let constant_type_token: proc_macro2::TokenStream = constant_type.parse().unwrap();

    let old_struct_ident = old_struct.ident.clone();

    let mut root_struct_name = old_struct.ident.to_string();
    root_struct_name.insert_str(0, "Root");
    let root_struct_ident = Ident::new(&root_struct_name, old_struct.ident.span());

    let mut subsonic_struct_name = old_struct.ident.to_string();
    subsonic_struct_name.insert_str(0, "Subsonic");
    let subsonic_struct_ident = Ident::new(&subsonic_struct_name, old_struct.ident.span());

    let mut json_type = old_struct_ident.to_string();
    json_type = match json_type.strip_suffix("Body") {
        Some(result) => result.to_owned(),
        _ => {
            return syn::Error::new(
                old_struct_ident.span(),
                "struct's name should end with `Body`",
            )
            .to_compile_error()
            .into();
        }
    };
    let json_type_ident = Ident::new(
        &concat_string!(json_type, "Response"),
        old_struct.ident.span(),
    );

    quote! {
        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        #old_struct

        #[derive(serde::Serialize)]
        pub struct #root_struct_ident {
            #[serde(flatten)]
            constant: #constant_type_token,
            #[serde(flatten)]
            body: #old_struct_ident,
        }

        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        pub struct #subsonic_struct_ident {
            #[serde(rename = "subsonic-response")]
            root: #root_struct_ident
        }

        pub type #json_type_ident = axum::Json<#subsonic_struct_ident>;

        impl From<#old_struct_ident> for #subsonic_struct_ident {
            fn from(old: #old_struct_ident) -> Self {
                Self {
                    root: #root_struct_ident {
                        constant: Default::default(),
                        body: old,
                    }
                }
            }
        }

        impl From<#old_struct_ident> for #json_type_ident {
            fn from(old: #old_struct_ident) -> Self {
                Self(old.into())
            }
        }
    }
    .into()
}

#[derive(Debug, FromMeta)]
struct AddValidateResponse {
    #[darling(default = "return_false")]
    admin: bool,
}

#[proc_macro_attribute]
pub fn add_validate(args: TokenStream, input: TokenStream) -> TokenStream {
    let item_struct = parse_macro_input!(input as ItemStruct);
    let item_struct_ident = item_struct.ident.clone();

    let attr_args = match NestedMeta::parse_meta_list(args.into()) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(Error::from(e).write_errors());
        }
    };
    let _args = match AddValidateResponse::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };

    let need_admin_token: proc_macro2::TokenStream = (if _args.admin { "true" } else { "false" })
        .parse()
        .unwrap();

    let validated_form_token: proc_macro2::TokenStream =
        concat_string!(COMMON_REQUEST_IMPORT_PREFIX, "::ValidatedForm")
            .parse()
            .unwrap();

    let mut validated_type = item_struct_ident.to_string();
    validated_type = match validated_type.strip_suffix("Params") {
        Some(result) => result.to_owned(),
        _ => {
            return syn::Error::new(
                item_struct_ident.span(),
                "struct's name should end with `Params`",
            )
            .to_compile_error()
            .into();
        }
    };
    let validated_type_ident = Ident::new(
        &concat_string!(validated_type, "Request"),
        item_struct_ident.span(),
    );

    quote!(
        #[derive(serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        #item_struct

        pub type #validated_type_ident = #validated_form_token<#item_struct_ident, #need_admin_token>;
    )
    .into()
}

fn return_true() -> bool {
    true
}

fn return_false() -> bool {
    false
}
