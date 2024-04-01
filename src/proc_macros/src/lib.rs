#![deny(clippy::all)]
#![feature(try_blocks)]

use std::ops::{Deref, DerefMut};

use concat_string::concat_string;
use darling::ast::NestedMeta;
use darling::FromMeta;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parser;
use syn::spanned::Spanned;
use syn::{parse_macro_input, parse_quote, Error, Expr, Ident, ItemStruct};

const CONSTANT_RESPONSE_IMPORT_PREFIX: &str = "crate::open_subsonic::common::response";
const COMMON_REQUEST_IMPORT_PREFIX: &str = "crate::open_subsonic::common::request";
const COMMON_ERROR_IMPORT_PREFIX: &str = "crate::open_subsonic::common::error";

#[derive(Debug, FromMeta)]
struct WrapSubsonicResponse {
    #[darling(default = "return_true")]
    success: bool,
}

#[proc_macro_attribute]
pub fn wrap_subsonic_response(args: TokenStream, input: TokenStream) -> TokenStream {
    let item_struct = parse_macro_input!(input as ItemStruct);
    let item_struct_ident = &item_struct.ident;

    let args = match try {
        let attr_args = NestedMeta::parse_meta_list(args.into())?;
        WrapSubsonicResponse::from_list(&attr_args)?
    } {
        Ok(r) => r,
        Err::<_, Error>(e) => return e.to_compile_error().into(),
    };

    let constant_type = if args.success {
        concat_string!(CONSTANT_RESPONSE_IMPORT_PREFIX, "::SuccessConstantResponse")
    } else {
        concat_string!(CONSTANT_RESPONSE_IMPORT_PREFIX, "::ErrorConstantResponse")
    };
    let constant_type_token: proc_macro2::TokenStream = constant_type.parse().unwrap();

    let root_struct_ident = Ident::new(
        &concat_string!("Root", item_struct_ident.to_string()),
        item_struct_ident.span(),
    );

    let subsonic_struct_ident = Ident::new(
        &concat_string!("Subsonic", item_struct_ident.to_string()),
        item_struct_ident.span(),
    );

    let base_type = match item_struct_ident.to_string().strip_suffix("Body") {
        Some(result) => result.to_owned(),
        _ => {
            return Error::new(item_struct_ident.span(), "struct's name should end with `Body`")
                .to_compile_error()
                .into();
        }
    };

    let json_response_type_token: proc_macro2::TokenStream =
        concat_string!(COMMON_ERROR_IMPORT_PREFIX, "::ServerJsonResponse").parse().unwrap();
    let json_response_type_ident =
        Ident::new(&concat_string!(base_type, "JsonResponse"), item_struct_ident.span());

    quote! {
        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        #item_struct

        #[derive(serde::Serialize)]
        pub struct #root_struct_ident {
            #[serde(flatten)]
            constant: #constant_type_token,
            #[serde(flatten)]
            body: #item_struct_ident,
        }

        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        pub struct #subsonic_struct_ident {
            #[serde(rename = "subsonic-response")]
            root: #root_struct_ident
        }

        pub type #json_response_type_ident = #json_response_type_token<#subsonic_struct_ident>;

        impl From<#item_struct_ident> for #subsonic_struct_ident {
            fn from(old: #item_struct_ident) -> Self {
                Self {
                    root: #root_struct_ident {
                        constant: Default::default(),
                        body: old,
                    }
                }
            }
        }

        impl From<#item_struct_ident> for #json_response_type_ident {
            fn from(old: #item_struct_ident) -> Self {
                Ok(axum::Json(old.into()))
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
    let item_struct_ident = &item_struct.ident;

    let mut validate_item_struct = item_struct.clone();

    let args = match try {
        let attr_args = NestedMeta::parse_meta_list(args.into())?;
        AddValidateResponse::from_list(&attr_args)?
    } {
        Ok(r) => r,
        Err::<_, Error>(e) => return e.to_compile_error().into(),
    };

    let need_admin_token: proc_macro2::TokenStream =
        (if args.admin { "true" } else { "false" }).parse().unwrap();

    let params_fields = if let syn::Fields::Named(ref fields) = item_struct.fields {
        match fields
            .named
            .iter()
            .map(|f| {
                f.ident
                    .as_ref()
                    .map(|ident| {
                        quote! {
                            #ident: self.#ident
                        }
                    })
                    .ok_or(Error::new(f.span(), "struct field name is missing"))
            })
            .collect::<Result<_, Error>>()
        {
            Ok(r) => r,
            Err(e) => return e.to_compile_error().into(),
        }
    } else {
        vec![]
    };

    let common_type_token: proc_macro2::TokenStream =
        concat_string!(COMMON_REQUEST_IMPORT_PREFIX, "::CommonParams").parse().unwrap();
    let validate_trait_token: proc_macro2::TokenStream =
        concat_string!(COMMON_REQUEST_IMPORT_PREFIX, "::Validate").parse().unwrap();
    let validated_form_token: proc_macro2::TokenStream =
        concat_string!(COMMON_REQUEST_IMPORT_PREFIX, "::ValidatedForm").parse().unwrap();

    let validated_type = match item_struct_ident.to_string().strip_suffix("Params") {
        Some(some) => some.to_owned(),
        _ => {
            return Error::new(item_struct_ident.span(), "struct's name should end with `Params`")
                .to_compile_error()
                .into();
        }
    };
    let validated_form_ident =
        Ident::new(&concat_string!(validated_type, "Request"), item_struct_ident.span());

    validate_item_struct.ident =
        Ident::new(&concat_string!(validated_type, "Validate"), item_struct_ident.span());
    let validate_item_ident = &validate_item_struct.ident;
    if let syn::Fields::Named(ref mut fields) = validate_item_struct.fields {
        fields.named.push(
            syn::Field::parse_named
                .parse2(quote! {
                    #[serde(flatten)]
                    pub common: #common_type_token
                })
                .unwrap(),
        );
    };
    let mut validate_params_fields = params_fields.clone();
    validate_params_fields.push(quote! { common, });

    quote!(
        #[derive(serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        #item_struct

        #[derive(serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        #validate_item_struct

        impl #validate_trait_token<#item_struct_ident> for #validate_item_ident {
            fn common(&self) -> &#common_type_token {
                &self.common
            }

            fn params(self) -> #item_struct_ident {
                #item_struct_ident {
                    #( #params_fields ),*
                }
            }
        }

        pub type #validated_form_ident =
            #validated_form_token<#validate_item_ident, #item_struct_ident, #need_admin_token>;

        #[cfg(test)]
        impl #item_struct_ident {
            fn to_validate(self, common: #common_type_token) -> #validate_item_ident {
                #validate_item_ident {
                    #( #validate_params_fields ),*
                }
            }

            fn to_validated_form(self, user_id: uuid::Uuid) -> #validated_form_ident {
                #validated_form_ident {
                    params: self,
                    user_id,
                    phantom: std::marker::PhantomData,
                }
            }
        }
    )
    .into()
}

#[proc_macro_attribute]
pub fn add_permission_filter(_: TokenStream, input: TokenStream) -> TokenStream {
    let expr = syn::parse::<Expr>(input).unwrap();

    let filters = vec![
        quote! {songs::music_folder_id.eq_any(music_folder_ids)},
        quote! {crate::open_subsonic::common::music_folder::with_music_folders(user_id)},
    ];
    let mut filter_exprs = match filters
        .into_iter()
        .map(|f| {
            let mut expr = expr.clone();
            let mut current_expr = &mut expr;
            loop {
                match current_expr {
                    Expr::MethodCall(ref mut expr) => {
                        let receiver_expr = expr.receiver.deref();
                        if let Expr::Call(head_expr) = receiver_expr {
                            expr.receiver =
                                Box::new(Expr::MethodCall(parse_quote! {#head_expr.filter(#f)}));
                            break;
                        } else {
                            current_expr = expr.receiver.deref_mut();
                        }
                    }
                    expr => {
                        return Err(Error::new(
                            expr.span(),
                            "item in expression should be a function call",
                        ));
                    }
                }
            }
            Ok(expr)
        })
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(r) => r,
        Err(e) => return e.to_compile_error().into(),
    };

    let filter_expr_with_user_id = filter_exprs.pop();
    let filter_expr_with_music_folder_ids = filter_exprs.pop();
    quote!(if let Some(music_folder_ids) = music_folder_ids.as_ref() {
        #filter_expr_with_music_folder_ids
    } else {
        #filter_expr_with_user_id
    })
    .into()
}

fn return_true() -> bool {
    true
}

fn return_false() -> bool {
    false
}
