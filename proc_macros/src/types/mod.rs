use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse::Parser;
use syn::spanned::Spanned;
use syn::{Error, Field, Fields, Ident, ItemStruct, Type};

use crate::all_roles;

#[derive(Debug, deluxe::ParseMetaItem)]
struct AddConvertTypes {
    #[deluxe(default)]
    from: Option<TokenStream>,
    #[deluxe(default)]
    into: Option<TokenStream>,
    #[deluxe(default)]
    both: Option<TokenStream>,
    #[deluxe(default)]
    skips: HashSet<Ident>,
    #[deluxe(default)]
    refs: HashSet<Ident>,
}

fn type_to_string(ty: &Type) -> Result<String, Error> {
    if let Type::Path(p) = ty {
        Ok(p.path
            .segments
            .last()
            .ok_or(Error::new(p.span(), "last path segment is missing"))?
            .ident
            .to_string())
    } else {
        Err(Error::new(ty.span(), "type should be a path"))
    }
}

fn type_is_integer(ty: &Type) -> Result<bool, Error> {
    let ty = type_to_string(ty)?;
    if ty.starts_with('u') || ty.starts_with('i') {
        Ok(ty[1..].parse::<u8>().is_ok())
    } else {
        Ok(false)
    }
}

pub fn add_types_derive(input: TokenStream) -> Result<TokenStream, Error> {
    Ok(quote! {
        #[derive(serde::Serialize, serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        #input
    })
}

pub fn add_role_fields(input: TokenStream) -> Result<TokenStream, Error> {
    let mut item_struct: ItemStruct = syn::parse2(input)?;
    if let Fields::Named(ref mut fields) = item_struct.fields {
        all_roles().iter().for_each(|r| {
            let role_name = format_ident!("{}_role", r);
            fields.named.push(
                Field::parse_named
                    .parse2(quote! {
                        pub #role_name: bool
                    })
                    .unwrap(),
            );
        })
    }
    Ok(quote! {
        #item_struct
    })
}

pub fn add_convert_types(args: TokenStream, input: TokenStream) -> Result<TokenStream, Error> {
    let item_struct: ItemStruct = syn::parse2(input)?;
    let item_ident = &item_struct.ident;
    let args = deluxe::parse2::<AddConvertTypes>(args)?;

    let params_fields = if let Fields::Named(ref fields) = item_struct.fields {
        fields
            .named
            .iter()
            .map(|f| {
                f.ident.as_ref().and_then(|ident| {
                    if !args.skips.contains(ident) {
                        if args.refs.contains(ident) {
                            Some(quote! { #ident: (&value.#ident).into() })
                        } else if type_is_integer(&f.ty).ok()? {
                            Some(quote! { #ident: value.#ident as _ })
                        } else {
                            Some(quote! { #ident: value.#ident.into() })
                        }
                    } else {
                        None
                    }
                })
            })
            .collect::<Vec<_>>()
    } else {
        vec![]
    };

    let (impl_generics, ty_generics, where_clause) = item_struct.generics.split_for_impl();

    let from = args.from.or(args.both.clone());
    let into = args.into.or(args.both.clone());

    let from_impl = if let Some(from) = from {
        quote! {
            impl #impl_generics From<#from> for #item_ident #ty_generics #where_clause {
                fn from(value: #from) -> Self {
                    Self {
                        #( #params_fields ),*
                    }
                }
            }
        }
    } else {
        quote! {}
    };

    let into_impl = if let Some(into) = into {
        quote! {
            impl #impl_generics From<#item_ident #ty_generics> for #into #where_clause {
                fn from(value: #item_ident) -> Self {
                    Self {
                        #( #params_fields ),*
                    }
                }
            }
        }
    } else {
        quote! {}
    };

    Ok(quote! {
        #item_struct

        #from_impl

        #into_impl
    })
}
