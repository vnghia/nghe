#![deny(clippy::all)]
#![feature(let_chains)]
#![feature(try_blocks, yeet_expr)]
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};
use std::sync::OnceLock;

use concat_string::concat_string;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::parse::Parser;
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, parse_quote, parse_str, Error, Expr, ExprPath, Field, Fields, Item,
    ItemStruct,
};

const CONSTANT_RESPONSE_IMPORT_PREFIX: &str = "crate::open_subsonic::common::response";
const COMMON_REQUEST_IMPORT_PREFIX: &str = "crate::open_subsonic::common::request";
const COMMON_ERROR_IMPORT_PREFIX: &str = "crate::open_subsonic::common::error";

#[derive(deluxe::ParseMetaItem)]
struct WrapSubsonicResponse {
    #[deluxe(default = true)]
    success: bool,
}

#[proc_macro_attribute]
pub fn wrap_subsonic_response(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    match try {
        let item_struct = parse_macro_input!(input as ItemStruct);
        let item_struct_ident = &item_struct.ident;

        let args = deluxe::parse2::<WrapSubsonicResponse>(args.into())?;
        let constant_type = parse_str::<ExprPath>(&if args.success {
            concat_string!(CONSTANT_RESPONSE_IMPORT_PREFIX, "::SuccessConstantResponse")
        } else {
            concat_string!(CONSTANT_RESPONSE_IMPORT_PREFIX, "::ErrorConstantResponse")
        })?;

        let root_struct = format_ident!("Root{}", item_struct_ident.to_string());
        let subsonic_struct = format_ident!("Subsonic{}", item_struct_ident.to_string());

        let base_type = item_struct_ident
            .to_string()
            .strip_suffix("Body")
            .ok_or_else(|| {
                Error::new(item_struct_ident.span(), "struct's name should end with `Body`")
            })?
            .to_string();

        let json_response_path = parse_str::<ExprPath>(&concat_string!(
            COMMON_ERROR_IMPORT_PREFIX,
            "::ServerJsonResponse"
        ))?;
        let json_response_type = format_ident!("{}JsonResponse", base_type);

        quote! {
            #[derive(serde::Serialize)]
            #[serde(rename_all = "camelCase")]
            #item_struct

            #[derive(serde::Serialize)]
            pub struct #root_struct {
                #[serde(flatten)]
                constant: #constant_type,
                #[serde(flatten)]
                body: #item_struct_ident,
            }

            #[derive(serde::Serialize)]
            #[serde(rename_all = "camelCase")]
            pub struct #subsonic_struct {
                #[serde(rename = "subsonic-response")]
                root: #root_struct
            }

            pub type #json_response_type = #json_response_path<#subsonic_struct>;

            impl From<#item_struct_ident> for #subsonic_struct {
                fn from(old: #item_struct_ident) -> Self {
                    Self {
                        root: #root_struct {
                            constant: Default::default(),
                            body: old,
                        }
                    }
                }
            }

            impl From<#item_struct_ident> for #json_response_type {
                fn from(old: #item_struct_ident) -> Self {
                    Ok(axum::Json(old.into()))
                }
            }
        }
        .into()
    } {
        Ok(r) => r,
        Err::<_, Error>(e) => e.into_compile_error().into(),
    }
}

fn all_roles() -> &'static HashSet<String> {
    static ALL_ROLES: OnceLock<HashSet<String>> = OnceLock::new();
    ALL_ROLES.get_or_init(|| {
        let file = syn::parse_file(
            &std::fs::read_to_string(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .parent()
                    .unwrap()
                    .join("src")
                    .join("models")
                    .join("users.rs"),
            )
            .unwrap(),
        )
        .unwrap();
        let role_struct = file
            .items
            .into_iter()
            .find_map(|i| {
                if let Item::Struct(item) = i
                    && item.ident == "Role"
                {
                    Some(item)
                } else {
                    None
                }
            })
            .unwrap();
        if let Fields::Named(fields) = role_struct.fields {
            fields
                .named
                .into_iter()
                .map(|n| n.ident.unwrap().to_string().strip_suffix("_role").unwrap().to_string())
                .collect::<HashSet<_>>()
        } else {
            unreachable!()
        }
    })
}

#[derive(Debug, deluxe::ParseMetaItem)]
#[deluxe(transparent(flatten_unnamed, append))]
struct AddValidateResponse {
    roles: Vec<Expr>,
}

#[proc_macro_attribute]
pub fn add_validate(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    match try {
        let item_struct = parse_macro_input!(input as ItemStruct);
        let item_struct_ident = &item_struct.ident;
        let mut validate_item_struct = item_struct.clone();

        let args = deluxe::parse2::<AddValidateResponse>(args.into())?;
        let roles = args
            .roles
            .into_iter()
            .map(|e| {
                if let Expr::Path(p) = e {
                    Ok(p.path
                        .segments
                        .last()
                        .ok_or(Error::new(p.span(), "last path segment is missing"))?
                        .ident
                        .to_string())
                } else {
                    Err(Error::new(e.span(), "expression should be a path"))
                }
            })
            .collect::<Result<HashSet<_>, _>>()?;
        if !all_roles().is_superset(&roles) {
            do yeet Error::new(Span::call_site(), "inputs contain invalid role");
        }
        let role_stmts = all_roles()
            .iter()
            .map(|r| {
                let role_name = format_ident!("{}_role", r);
                let has_role = roles.contains(r);
                quote! { #role_name: #has_role }
            })
            .collect::<Vec<_>>();
        let role_struct_path = parse_str::<ExprPath>("crate::models::users::Role")?;
        let role_struct = quote! {
            #role_struct_path {
              #( #role_stmts ),*
            }
        };

        let params_fields = if let Fields::Named(ref fields) = item_struct.fields {
            fields
                .named
                .iter()
                .map(|f| {
                    f.ident
                        .as_ref()
                        .map(|ident| {
                            quote! { #ident: self.#ident }
                        })
                        .ok_or(Error::new(f.span(), "struct field name is missing"))
                })
                .collect::<Result<_, Error>>()?
        } else {
            vec![]
        };

        let common_path =
            parse_str::<ExprPath>(&concat_string!(COMMON_REQUEST_IMPORT_PREFIX, "::CommonParams"))?;
        let validate_trait_path =
            parse_str::<ExprPath>(&concat_string!(COMMON_REQUEST_IMPORT_PREFIX, "::Validate"))?;
        let validated_form_path = parse_str::<ExprPath>(&concat_string!(
            COMMON_REQUEST_IMPORT_PREFIX,
            "::ValidatedForm"
        ))?;

        let base_type = item_struct_ident
            .to_string()
            .strip_suffix("Params")
            .ok_or_else(|| {
                Error::new(item_struct_ident.span(), "struct's name should end with `Params`")
            })?
            .to_string();

        let validated_form = format_ident!("{}Request", base_type);
        validate_item_struct.ident = format_ident!("{}Validate", base_type);
        let validate_item_ident = &validate_item_struct.ident;
        if let Fields::Named(ref mut fields) = validate_item_struct.fields {
            fields.named.push(Field::parse_named.parse2(quote! {
                #[serde(flatten)]
                pub common: #common_path
            })?);
        };
        let mut validate_params_fields = params_fields.clone();
        validate_params_fields.push(quote! { common, });

        quote! {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            #item_struct

            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            #validate_item_struct

            impl #validate_trait_path<#item_struct_ident> for #validate_item_ident {
                fn common(&self) -> &#common_path {
                    &self.common
                }

                fn params(self) -> #item_struct_ident {
                    #item_struct_ident {
                        #( #params_fields ),*
                    }
                }
            }

            pub type #validated_form =
                #validated_form_path<#validate_item_ident, #item_struct_ident,{ #role_struct }>;

            #[cfg(test)]
            impl #item_struct_ident {
                fn to_validate(self, common: #common_path) -> #validate_item_ident {
                    #validate_item_ident {
                        #( #validate_params_fields ),*
                    }
                }

                fn to_validated_form(self, user_id: uuid::Uuid) -> #validated_form {
                    #validated_form {
                        params: self,
                        user_id,
                        phantom: std::marker::PhantomData,
                    }
                }
            }
        }
        .into()
    } {
        Ok(r) => r,
        Err::<_, Error>(e) => e.into_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn add_permission_filter(
    _: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    match try {
        let expr = syn::parse::<Expr>(input).unwrap();

        let filters: Vec<Expr> = vec![
            parse_quote! { songs::music_folder_id.eq_any(music_folder_ids) },
            parse_quote! { crate::open_subsonic::permission::with_permission(user_id) },
        ];

        let mut filter_exprs = filters
            .into_iter()
            .map(|f| {
                let mut expr = expr.clone();
                let mut current_expr = &mut expr;
                loop {
                    match current_expr {
                        Expr::MethodCall(ref mut expr) => {
                            let receiver_expr = expr.receiver.deref();
                            if let Expr::Call(head_expr) = receiver_expr {
                                expr.receiver = Box::new(Expr::MethodCall(
                                    parse_quote! {#head_expr.filter(#f)},
                                ));
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
            .collect::<Result<Vec<_>, _>>()?;

        let filter_expr_with_user_id = filter_exprs.pop();
        let filter_expr_with_music_folder_ids = filter_exprs.pop();
        quote!(
            if let Some(music_folder_ids) = music_folder_ids.as_ref() {
                #filter_expr_with_music_folder_ids
            } else {
                #filter_expr_with_user_id
            }
        )
        .into()
    } {
        Ok(r) => r,
        Err::<_, Error>(e) => e.into_compile_error().into(),
    }
}
