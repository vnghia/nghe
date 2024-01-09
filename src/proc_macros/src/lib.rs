use darling::{ast::NestedMeta, Error, FromMeta};
use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse::Parser, parse_macro_input, Ident, ItemStruct};

const CONSTANT_RESPONSE_IMPORT_PREFIX: &'static str = "crate::open_subsonic::common::response";
const COMMON_REQUEST_IMPORT_PREFIX: &'static str = "crate::open_subsonic::common::request";

#[derive(Debug, FromMeta)]
struct WrapSubsonicResponse {
    #[darling(default = "return_true")]
    success: bool,
}

#[proc_macro_attribute]
pub fn wrap_subsonic_response(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut old_struct = parse_macro_input!(input as ItemStruct);
    let mut new_struct = old_struct.clone();

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
        format!(
            "{}::SuccessConstantResponse",
            CONSTANT_RESPONSE_IMPORT_PREFIX
        )
    } else {
        format!("{}::ErrorConstantResponse", CONSTANT_RESPONSE_IMPORT_PREFIX)
    };
    let constant_type_token: proc_macro2::TokenStream = constant_type.parse().unwrap();

    let mut new_struct_name = new_struct.ident.to_string();
    new_struct_name.insert_str(0, "Wrapped");
    new_struct.ident = Ident::new(&new_struct_name, new_struct.ident.span());

    let old_struct_ident = old_struct.ident.clone();
    let new_struct_ident = new_struct.ident.clone();

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
    let json_type_ident = Ident::new(&format!("{}Response", json_type), old_struct.ident.span());

    if let syn::Fields::Named(ref mut old_fields) = old_struct.fields {
        if let syn::Fields::Named(ref mut new_fields) = new_struct.fields {
            old_fields.named.push(
                syn::Field::parse_named
                    .parse2(quote! {
                        #[serde(flatten)]
                        constant: #constant_type_token
                    })
                    .unwrap(),
            );
            new_fields.named.clear();
            new_fields.named.push(
                syn::Field::parse_named
                    .parse2(quote! {
                        #[serde(rename = "subsonic-response")]
                        root: #old_struct_ident
                    })
                    .unwrap(),
            )
        }
    }

    return quote! {
        #old_struct

        #new_struct

        pub type #json_type_ident = axum::Json<#new_struct_ident>;

        impl From<#old_struct_ident> for #new_struct_ident {
            fn from(old: #old_struct_ident) -> Self {
                Self {
                    root: old
                }
            }
        }

        impl From<#old_struct_ident> for #json_type_ident {
            fn from(old: #old_struct_ident) -> Self {
                Self(old.into())
            }
        }
    }
    .into();
}

#[derive(Debug, FromMeta)]
struct AddValidateResponse {
    #[darling(default = "return_false")]
    admin: bool,
}

#[proc_macro_attribute]
pub fn add_validate(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item_struct = parse_macro_input!(input as ItemStruct);
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

    let common_type_token: proc_macro2::TokenStream =
        format!("{}::CommonParams", COMMON_REQUEST_IMPORT_PREFIX)
            .parse()
            .unwrap();
    let validate_trait_token: proc_macro2::TokenStream =
        format!("{}::Validate", COMMON_REQUEST_IMPORT_PREFIX)
            .parse()
            .unwrap();
    let validated_form_token: proc_macro2::TokenStream =
        format!("{}::ValidatedForm", COMMON_REQUEST_IMPORT_PREFIX)
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
        &format!("{}Request", validated_type),
        item_struct_ident.span(),
    );

    if let syn::Fields::Named(ref mut fields) = item_struct.fields {
        fields.named.push(
            syn::Field::parse_named
                .parse2(quote! {
                    #[serde(flatten)]
                    common: #common_type_token
                })
                .unwrap(),
        );
    }

    return quote!(
        #item_struct

        impl #validate_trait_token for #item_struct_ident {
            fn get_common_params(&self) -> &#common_type_token {
                &self.common
            }

            #[inline(always)]
            fn need_admin(&self) -> bool {
                #need_admin_token
            }
        }

        pub type #validated_type_ident = #validated_form_token<#item_struct_ident>;
    )
    .into();
}

fn return_true() -> bool {
    return true;
}

fn return_false() -> bool {
    return false;
}
