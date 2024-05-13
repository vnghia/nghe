use std::ops::{Deref, DerefMut};

use concat_string::concat_string;
use convert_case::{Case, Casing};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{parse_quote, Error, Expr, ExprCall};

const DATE_TYPE_PREFIXES: &[&str] = &["", "release_", "original_release_"];

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

pub fn add_permission_filter(input: TokenStream) -> Result<TokenStream, Error> {
    let expr: Expr = syn::parse2(input)?;

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
    Ok(quote! {
        if let Some(music_folder_ids) = music_folder_ids.as_ref() {
            #filter_expr_with_music_folder_ids
        } else {
            #filter_expr_with_user_id
        }
    })
}

pub fn add_count_offset(args: TokenStream, input: TokenStream) -> Result<TokenStream, Error> {
    let prefix =
        if args.is_empty() { args.to_string() } else { concat_string!(args.to_string(), "_") };
    let count = format_ident!("{}count", &prefix);
    let offset = format_ident!("{}offset", &prefix);

    let mut expr: Expr = syn::parse2(input).unwrap();
    modify_head_call_expr(&mut expr, |head_expr| {
        Expr::MethodCall(parse_quote! {
            #head_expr.limit(#count as _).offset(#offset as _)
        })
    })?;

    Ok(quote! {
        #expr
    })
}

pub fn generate_date_db(table_name: TokenStream) -> Result<TokenStream, Error> {
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

    Ok(quote! {
        #( #date_structs ) *
    })
}
