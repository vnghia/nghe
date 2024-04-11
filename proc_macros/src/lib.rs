#![deny(clippy::all)]
#![feature(let_chains)]
#![feature(try_blocks, yeet_expr)]
#![feature(proc_macro_span)]

use std::collections::HashSet;
use std::ops::{Deref, DerefMut};
use std::sync::OnceLock;

use concat_string::concat_string;
use convert_case::{Case, Casing};
use proc_macro2::Span;
use quote::{format_ident, quote, ToTokens};
use syn::parse::Parser;
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, parse_quote, parse_str, Error, Expr, ExprCall, ExprPath, Field, Fields,
    GenericParam, Ident, Item, ItemStruct, Lifetime, LifetimeParam,
};

const TYPES_CONSTANT_RESPONSE_IMPORT_PREFIX: &str = "crate::response";
const TYPES_COMMON_PARAMS_IMPORT_PREFIX: &str = "crate::params";

const BACKEND_COMMON_PARAMS_IMPORT_PREFIX: &str = "crate::open_subsonic::params";
const BACKEND_COMMON_ERROR_IMPORT_PREFIX: &str = "crate::open_subsonic::common::error";

const DATE_TYPE_PREFIXES: &[&str] = &["", "release_", "original_release_"];

fn get_base_name(ident: &Ident, suffix: &str) -> Result<String, Error> {
    ident
        .to_string()
        .strip_suffix(suffix)
        .ok_or_else(|| {
            Error::new(ident.span(), concat_string!("struct's name should end with ", suffix))
        })
        .map(String::from)
}
fn get_caller_types_module() -> ExprPath {
    parse_str::<ExprPath>(
        &std::path::Path::new("nghe_types")
            .join(
                proc_macro::Span::call_site()
                    .source_file()
                    .path()
                    .with_extension("")
                    .strip_prefix("src")
                    .unwrap()
                    .strip_prefix("open_subsonic")
                    .unwrap(),
            )
            .components()
            .map(|c| {
                if let std::path::Component::Normal(c) = c {
                    c.to_string_lossy()
                } else {
                    unreachable!()
                }
            })
            .collect::<Vec<_>>()
            .join("::"),
    )
    .unwrap()
}

#[derive(deluxe::ParseMetaItem)]
struct AddSubsonicResponse {
    #[deluxe(default = true)]
    success: bool,
}

#[proc_macro_attribute]
pub fn add_subsonic_response(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    match try {
        let item_struct = parse_macro_input!(input as ItemStruct);
        let item_ident = &item_struct.ident;

        let args = deluxe::parse2::<AddSubsonicResponse>(args.into())?;
        let constant_type = parse_str::<ExprPath>(&if args.success {
            concat_string!(TYPES_CONSTANT_RESPONSE_IMPORT_PREFIX, "::SuccessConstantResponse")
        } else {
            concat_string!(TYPES_CONSTANT_RESPONSE_IMPORT_PREFIX, "::ErrorConstantResponse")
        })?;

        let root_ident = format_ident!("Root{}", item_ident.to_string());
        let subsonic_ident = format_ident!("Subsonic{}", item_ident.to_string());

        quote! {
            #[nghe_proc_macros::add_types_derive]
            #item_struct

            #[nghe_proc_macros::add_types_derive]
            pub struct #root_ident {
                #[serde(flatten)]
                pub constant: #constant_type,
                #[serde(flatten)]
                pub body: #item_ident,
            }

            #[nghe_proc_macros::add_types_derive]
            pub struct #subsonic_ident {
                #[serde(rename = "subsonic-response")]
                pub root: #root_ident
            }

            impl From<#item_ident> for #subsonic_ident {
                fn from(body: #item_ident) -> Self {
                    Self {
                        root: #root_ident {
                            constant: Default::default(),
                            body,
                        }
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

#[proc_macro]
pub fn add_axum_response(item_ident: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match try {
        let types_path = get_caller_types_module();

        let item_ident = format_ident!("{}", item_ident.to_string());
        let subsonic_ident = format_ident!("Subsonic{}", item_ident.to_string());

        let base_type = get_base_name(&item_ident, "Body")?;
        let json_response_path = parse_str::<ExprPath>(&concat_string!(
            BACKEND_COMMON_ERROR_IMPORT_PREFIX,
            "::ServerJsonResponse"
        ))?;
        let json_response_ident = format_ident!("{}JsonResponse", base_type);

        quote! {
            use #types_path::*;

            pub type #json_response_ident = #json_response_path<#subsonic_ident>;
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

#[proc_macro_attribute]
pub fn add_common_convert(
    _: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    match try {
        let item_struct = parse_macro_input!(input as ItemStruct);
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
        let with_common_path = parse_str::<ExprPath>(&concat_string!(
            TYPES_COMMON_PARAMS_IMPORT_PREFIX,
            "::WithCommon"
        ))?;
        let base_type = get_base_name(item_ident, "Params")?;

        common_item_struct.ident = format_ident!("{}WithCommon", base_type);
        let common_item_ident = &common_item_struct.ident;

        let lt = Lifetime::new("'common", Span::call_site());
        common_item_struct
            .generics
            .params
            .push(GenericParam::Lifetime(LifetimeParam::new(lt.clone())));
        if let Fields::Named(ref mut fields) = common_item_struct.fields {
            fields.named.push(Field::parse_named.parse2(quote! {
                #[serde(flatten)]
                pub common: std::borrow::Cow<#lt, #common_path>
            })?);
        }

        let mut common_params_fields = params_fields.clone();
        common_params_fields.push(quote! { common });

        quote! {
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
        }
        .into()
    } {
        Ok(r) => r,
        Err::<_, Error>(e) => e.into_compile_error().into(),
    }
}

#[derive(Debug, deluxe::ParseMetaItem)]
#[deluxe(transparent(flatten_unnamed, append))]
struct AddCommonValidate {
    args: Vec<Expr>,
}

#[proc_macro]
pub fn add_common_validate(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match try {
        let types_path = get_caller_types_module();

        let mut args = deluxe::parse2::<AddCommonValidate>(input.into())?
            .args
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
            .collect::<Result<Vec<_>, _>>()?;

        let item_ident = format_ident!("{}", args.remove(0));

        let roles = HashSet::from_iter(args);
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

        let validated_form_path = parse_str::<ExprPath>(&concat_string!(
            BACKEND_COMMON_PARAMS_IMPORT_PREFIX,
            "::ValidatedForm"
        ))?;

        let base_type = get_base_name(&item_ident, "Params")?;

        let request_ident = format_ident!("{}Request", base_type);
        let common_item_ident = format_ident!("{}WithCommon", base_type);

        quote! {
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
        }
        .into()
    } {
        Ok(r) => r,
        Err::<_, Error>(e) => e.into_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn add_types_derive(
    _: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    match try {
        let input: proc_macro2::TokenStream = input.into();
        quote! {
            #[derive(serde::Serialize, serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            #input
        }
        .into()
    } {
        Ok(r) => r,
        Err::<_, Error>(e) => e.into_compile_error().into(),
    }
}

fn modify_head_call_expr<F>(expr: &mut Expr, new_head_call: F) -> Result<(), Error>
where
    F: Fn(&ExprCall) -> Expr,
{
    let mut current_expr = expr;
    loop {
        match current_expr {
            Expr::MethodCall(ref mut expr) => {
                let receiver_expr = expr.receiver.deref();
                if let Expr::Call(head_expr) = receiver_expr {
                    expr.receiver = Box::new(new_head_call(head_expr));
                    break;
                } else {
                    current_expr = expr.receiver.deref_mut();
                }
            }
            Expr::Await(ref mut expr) => {
                let base_expr = expr.base.deref();
                if let Expr::Call(head_expr) = base_expr {
                    expr.base = Box::new(new_head_call(head_expr));
                    break;
                } else {
                    current_expr = expr.base.deref_mut();
                }
            }
            Expr::Try(ref mut expr) => {
                let expr_expr = expr.expr.deref();
                if let Expr::Call(head_expr) = expr_expr {
                    expr.expr = Box::new(new_head_call(head_expr));
                    break;
                } else {
                    current_expr = expr.expr.deref_mut();
                }
            }
            expr => {
                do yeet Error::new(
                    expr.span(),
                    "item in expression should be a function call, await or try",
                )
            }
        }
    }

    Ok(())
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
                modify_head_call_expr(&mut expr, |head_expr| {
                    Expr::MethodCall(parse_quote! {#head_expr.filter(#f)})
                })?;
                Ok::<_, Error>(expr)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let filter_expr_with_user_id = filter_exprs.pop();
        let filter_expr_with_music_folder_ids = filter_exprs.pop();
        quote! {
            if let Some(music_folder_ids) = music_folder_ids.as_ref() {
                #filter_expr_with_music_folder_ids
            } else {
                #filter_expr_with_user_id
            }
        }
        .into()
    } {
        Ok(r) => r,
        Err::<_, Error>(e) => e.into_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn add_count_offset(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    match try {
        let prefix =
            if args.is_empty() { args.to_string() } else { concat_string!(args.to_string(), "_") };
        let count = format_ident!("{}count", &prefix);
        let offset = format_ident!("{}offset", &prefix);

        let mut expr = syn::parse::<Expr>(input).unwrap();
        modify_head_call_expr(&mut expr, |head_expr| {
            Expr::MethodCall(
                parse_quote! {#head_expr.limit(#count.unwrap_or(20)).offset(#offset.unwrap_or(0))},
            )
        })?;
        expr.into_token_stream().into()
    } {
        Ok(r) => r,
        Err::<_, Error>(e) => e.into_compile_error().into(),
    }
}

#[proc_macro]
pub fn generate_date_db(table_name: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match try {
        let table_name = table_name.to_string();
        let table_prefix = table_name
            .strip_suffix('s')
            .ok_or_else(|| Error::new(Span::call_site(), "table name should end with `s`"))?;
        let date_structs = DATE_TYPE_PREFIXES
            .iter()
            .map(|prefix| {
                let date_type = format_ident!(
                    "{}",
                    concat_string!(&table_prefix, "_", prefix, "date_db").to_case(Case::Pascal)
                );
                let table_name = format_ident!("{}", &table_name);

                let year_column = format_ident!("{}year", prefix);
                let month_column = format_ident!("{}month", prefix);
                let day_column = format_ident!("{}day", prefix);

                quote! {
                    #[derive(
                        Debug,
                        Clone,
                        Copy,
                        diesel::Queryable,
                        diesel::Selectable,
                        diesel::Insertable
                    )]
                    #[diesel(table_name = #table_name)]
                    #[diesel(check_for_backend(diesel::pg::Pg))]
                    #[cfg_attr(test, derive(Default, Hash, PartialEq, Eq, PartialOrd, Ord))]
                    pub struct #date_type {
                        #[diesel(column_name = #year_column)]
                        pub year: Option<i16>,
                        #[diesel(column_name = #month_column)]
                        pub month: Option<i16>,
                        #[diesel(column_name = #day_column)]
                        pub day: Option<i16>,
                    }

                    impl From<crate::utils::song::SongDate> for #date_type {
                        fn from(value: crate::utils::song::SongDate) -> Self {
                            let (y, m, d) = value.to_ymd();
                            Self { year: y, month: m, day: d }
                        }
                    }

                    impl Into<nghe_types::id3::DateId3> for #date_type {
                        fn into(self) -> nghe_types::id3::DateId3 {
                            nghe_types::id3::DateId3 {
                                year: self.year.map(|v| v as _),
                                month: self.month.map(|v| v as _),
                                day: self.day.map(|v| v as _),
                            }
                        }
                    }

                    #[cfg(test)]
                    impl From<#date_type> for crate::utils::song::SongDate {
                        fn from(value: #date_type) -> Self {
                            Self::from_ymd(value.year, value.month, value.day)
                        }
                    }
                }
            })
            .collect::<Vec<_>>();
        quote! {
            #( #date_structs ) *
        }
        .into()
    } {
        Ok(r) => r,
        Err::<_, Error>(e) => e.into_compile_error().into(),
    }
}
