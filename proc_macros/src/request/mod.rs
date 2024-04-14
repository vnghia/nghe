use std::collections::HashSet;

use concat_string::concat_string;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::parse::Parser;
use syn::spanned::Spanned;
use syn::{
    parse_str, Error, Expr, ExprPath, Field, Fields, GenericParam, ItemStruct, Lifetime,
    LifetimeParam,
};

use crate::{all_roles, expr_to_string, get_base_name, get_caller_types_module};

const TYPES_COMMON_PARAMS_IMPORT_PREFIX: &str = "crate::params";
const BACKEND_COMMON_PARAMS_IMPORT_PREFIX: &str = "crate::open_subsonic::params";

#[derive(Debug, deluxe::ParseMetaItem)]
#[deluxe(transparent(flatten_unnamed, append))]
struct AddCommonValidate {
    args: Vec<Expr>,
}

pub fn add_common_convert(input: TokenStream) -> Result<TokenStream, Error> {
    let item_struct: ItemStruct = syn::parse2(input)?;
    let item_ident = &item_struct.ident;
    let mut common_item_struct = item_struct.clone();

    let params_fields = if let Fields::Named(ref fields) = item_struct.fields {
        fields
            .named
            .iter()
            .map(|f| {
                f.ident
                    .as_ref()
                    .map(|ident| {
                        quote! { #ident: value.#ident }
                    })
                    .ok_or(Error::new(f.span(), "struct field name is missing"))
            })
            .collect::<Result<_, Error>>()?
    } else {
        vec![]
    };

    let common_path = parse_str::<ExprPath>(&concat_string!(
        TYPES_COMMON_PARAMS_IMPORT_PREFIX,
        "::CommonParams"
    ))?;
    let with_common_path =
        parse_str::<ExprPath>(&concat_string!(TYPES_COMMON_PARAMS_IMPORT_PREFIX, "::WithCommon"))?;
    let base_type = get_base_name(item_ident, "Params")?;

    common_item_struct.ident = format_ident!("{}WithCommon", base_type);
    let common_item_ident = &common_item_struct.ident;

    let lt = Lifetime::new("'common", Span::call_site());
    common_item_struct.generics.params.push(GenericParam::Lifetime(LifetimeParam::new(lt.clone())));
    if let Fields::Named(ref mut fields) = common_item_struct.fields {
        fields.named.push(Field::parse_named.parse2(quote! {
            #[serde(flatten)]
            pub common: std::borrow::Cow<#lt, #common_path>
        })?);
    }

    let mut common_params_fields = params_fields.clone();
    common_params_fields.push(quote! { common });

    Ok(quote! {
        #[nghe_proc_macros::add_types_derive]
        #item_struct

        #[nghe_proc_macros::add_types_derive]
        #common_item_struct

        impl AsRef<#common_path> for #common_item_ident<'static> {
            fn as_ref(&self) -> &#common_path {
                &self.common.as_ref()
            }
        }

        impl<#lt> From<#common_item_ident<#lt>> for #item_ident {
            fn from(value: #common_item_ident<#lt>) -> #item_ident {
                Self {
                    #( #params_fields ),*
                }
            }
        }

        impl<#lt> #with_common_path<#lt> for #item_ident {
            type Out = #common_item_ident<#lt>;

            fn with_common<T: Into<std::borrow::Cow<#lt, #common_path>>>(
                self, common: T
            ) -> #common_item_ident<#lt> {
                let value = self;
                let common = common.into();
                #common_item_ident {
                    #( #common_params_fields ),*
                }
            }
        }
    })
}

pub fn add_common_validate(input: TokenStream) -> Result<TokenStream, Error> {
    let types_path = get_caller_types_module();

    let mut args = deluxe::parse2::<AddCommonValidate>(input)?
        .args
        .iter()
        .map(expr_to_string)
        .collect::<Result<Vec<_>, _>>()?;

    let item_ident = format_ident!("{}", args.remove(0));

    let roles = HashSet::from_iter(args);
    if !all_roles().iter().cloned().collect::<HashSet<_>>().is_superset(&roles) {
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

    let validated_form_path = parse_str::<ExprPath>(&concat_string!(
        BACKEND_COMMON_PARAMS_IMPORT_PREFIX,
        "::ValidatedForm"
    ))?;

    let base_type = get_base_name(&item_ident, "Params")?;

    let request_ident = format_ident!("{}Request", base_type);
    let common_item_ident = format_ident!("{}WithCommon", base_type);

    Ok(quote! {
        use #types_path::*;

        pub type #request_ident =
            #validated_form_path<#common_item_ident<'static>, #item_ident, { #role_struct }>;

        #[cfg(test)]
        impl #request_ident {
            fn validated(
                params: #item_ident, user_id: uuid::Uuid, user_role: #role_struct_path
            ) -> Self {
                Self {
                    params,
                    user_id,
                    user_role,
                    phantom: std::marker::PhantomData,
                }
            }
        }
    })
}
