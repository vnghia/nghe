use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, parse_quote};

use crate::utils;

#[derive(Debug, darling::FromMeta)]
#[darling(derive_syn_parse)]
struct BuildRoute {
    routes: utils::syn::Vec<syn::Ident>,
}

pub fn build(item: TokenStream) -> Result<TokenStream, Error> {
    let BuildRoute { routes } = syn::parse2(item)?;
    let (variants, arms): (Vec<_>, Vec<_>) = routes
        .iter()
        .map(|ident| {
            let variant = syn::Ident::new(&ident.to_string().to_case(Case::Pascal), ident.span());
            Ok::<(syn::Variant, syn::Arm), Error>((
                parse_quote!(
                    #variant {
                        #[conf(subcommands)]
                        action: #ident::Action,
                    }
                ),
                parse_quote!(
                    Self::#variant { action } => action.run(runner).await
                ),
            ))
        })
        .try_collect()?;

    Ok(quote! {
        use conf::{Conf, Subcommands};
        use crate::command::rest::Runner;
        use crate::command::Error;

        #[derive(Debug, Subcommands)]
        pub enum Route {
            #( #variants ),*
        }

        impl Route {
            pub async fn run(&self, runner: &Runner) -> Result<(), Error> {
                match self {
                    #( #arms ),*
                }
            }
        }
    })
}
