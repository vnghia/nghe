use concat_string::concat_string;
use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_str, Error, ExprPath, ItemStruct};

const COMMON_PARAMS_IMPORT_PREFIX: &str = "crate::params";

fn get_method_path() -> String {
    proc_macro::Span::call_site()
        .source_file()
        .path()
        .with_extension("")
        .strip_prefix("lastfm-client")
        .unwrap()
        .strip_prefix("src")
        .unwrap()
        .components()
        .map(|c| {
            if let std::path::Component::Normal(c) = c {
                c.to_string_lossy()
            } else {
                unreachable!()
            }
        })
        .collect::<Vec<_>>()
        .join(".")
}

pub fn add_method_name(input: TokenStream) -> Result<TokenStream, Error> {
    let item_struct: ItemStruct = syn::parse2(input)?;
    let item_ident = &item_struct.ident;
    let method_name = get_method_path().to_case(Case::Flat);
    let method_name_trait =
        parse_str::<ExprPath>(&concat_string!(COMMON_PARAMS_IMPORT_PREFIX, "::MethodName"))?;

    Ok(quote! {
        impl #method_name_trait for #item_ident {
            fn method_name() -> &'static str {
                #method_name
            }
        }
    })
}
