use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, parse_quote};

use crate::utils;

#[derive(Debug, darling::FromMeta, bon::Builder)]
#[darling(derive_syn_parse)]
struct Endpoint {
    #[darling(default = || true)]
    #[builder(default = true)]
    body: bool,
    #[darling(default = || false)]
    #[builder(default = false)]
    binary: bool,
}

#[derive(Debug, darling::FromMeta)]
#[darling(derive_syn_parse)]
struct BuildCommand {
    module: syn::Ident,
    actions: utils::syn::Vec<syn::Meta>,
}

pub fn build(item: TokenStream) -> Result<TokenStream, Error> {
    let BuildCommand { module, actions } = syn::parse2(item)?;
    let (variants, arms): (Vec<_>, Vec<_>) = actions
        .iter()
        .map(|meta| {
            let action = meta.path().require_ident()?.to_owned();
            let endpoint = if let syn::Meta::List(syn::MetaList { tokens, .. }) = meta {
                syn::parse2(tokens.clone())?
            } else {
                Endpoint::builder().build()
            };

            let variant = syn::Ident::new(&action.to_string().to_case(Case::Pascal), module.span());
            Ok::<(syn::Variant, syn::Arm), Error>(if endpoint.body {
                (
                    parse_quote!(
                        #variant(nghe_api::route::#module::#action::Request)
                    ),
                    if endpoint.binary {
                        parse_quote!(
                            Self::#variant(request) => runner.run_binary(Some(request)).await
                        )
                    } else {
                        parse_quote!(
                            Self::#variant(request) => runner.run_endpoint(Some(request)).await
                        )
                    },
                )
            } else {
                (
                    parse_quote!(
                        #variant
                    ),
                    if endpoint.binary {
                        parse_quote!(
                            Self::#variant => runner.run_binary::<
                                nghe_api::route::#module::#action::Request
                            >(None).await
                        )
                    } else {
                        parse_quote!(
                            Self::#variant => runner.run_endpoint::<
                                nghe_api::route::#module::#action::Request
                            >(None).await
                        )
                    },
                )
            })
        })
        .try_collect()?;

    Ok(quote! {
        use conf::{Conf, Subcommands};
        use crate::command::rest::Runner;
        use crate::command::Error;

        #[derive(Debug, Subcommands)]
        pub enum Action {
            #( #variants ),*
        }

        impl Action {
            pub async fn run(&self, runner: &Runner) -> Result<(), Error> {
                match self {
                    #( #arms ),*
                }
            }
        }
    })
}
