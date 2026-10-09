use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, parse_quote};

use crate::utils;

#[derive(Debug, darling::FromMeta, bon::Builder)]
#[darling(derive_syn_parse)]
struct Attribute {
    #[darling(default = || true)]
    #[builder(default = true)]
    body: bool,
}

#[derive(Debug, darling::FromMeta)]
#[darling(derive_syn_parse)]
struct BuildCommand {
    module: syn::Ident,
    actions: utils::syn::Vec<syn::Meta>,
}

pub fn build(item: TokenStream) -> Result<TokenStream, Error> {
    let BuildCommand { module, actions } = syn::parse2(item)?;
    let variants: Vec<_> = actions
        .iter()
        .map(|meta| {
            let action = meta.path().require_ident()?.to_owned();
            let attribute = if let syn::Meta::List(syn::MetaList { tokens, .. }) = meta {
                syn::parse2(tokens.clone())?
            } else {
                Attribute::builder().build()
            };

            let variant = syn::Ident::new(&action.to_string().to_case(Case::Pascal), module.span());
            Ok::<syn::Variant, Error>(if attribute.body {
                parse_quote!(
                    #variant(nghe_api::route::#module::#action::Request)
                )
            } else {
                parse_quote!(
                    #variant
                )
            })
        })
        .try_collect()?;

    Ok(quote! {
        use conf::{Conf, Subcommands};

        #[derive(Debug, Subcommands)]
        pub enum Action {
            #( #variants ),*
        }
    })
}
