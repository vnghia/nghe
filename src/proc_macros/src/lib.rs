use darling::{ast::NestedMeta, Error, FromMeta};
use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse::Parser, parse_macro_input, Ident, ItemStruct};

#[derive(Debug, FromMeta)]
struct WrapSubsonicResponse {
    #[darling(default = "return_true")]
    success: bool,
}

#[proc_macro_attribute]
pub fn wrap_subsonic_response(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut old_struct = parse_macro_input!(input as ItemStruct);
    let mut new_struct = old_struct.clone();

    let attr_args = match NestedMeta::parse_meta_list(args.into()) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(Error::from(e).write_errors());
        }
    };
    let _args = match WrapSubsonicResponse::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };

    let constant_type = if _args.success {
        "SuccessConstantResponse"
    } else {
        "ErrorConstantResponse"
    };
    let constant_type_token: proc_macro2::TokenStream = constant_type.parse().unwrap();

    let mut old_struct_name = old_struct.ident.to_string();
    old_struct_name.insert_str(0, "Actual");
    old_struct.ident = Ident::new(&old_struct_name, old_struct.ident.span());

    let old_struct_name_token: proc_macro2::TokenStream = old_struct_name.parse().unwrap();

    if let syn::Fields::Named(ref mut old_fields) = old_struct.fields {
        if let syn::Fields::Named(ref mut new_fields) = new_struct.fields {
            old_fields.named.push(
                syn::Field::parse_named
                    .parse2(quote! {
                        #[serde(flatten)]
                        constant: #constant_type_token
                    })
                    .unwrap(),
            );
            new_fields.named.clear();
            new_fields.named.push(
                syn::Field::parse_named
                    .parse2(quote! {
                        #[serde(rename = "subsonic-response")]
                        subsonic_response: #old_struct_name_token
                    })
                    .unwrap(),
            )
        }
    }

    return quote! {
        #old_struct

        #new_struct
    }
    .into();
}

fn return_true() -> bool {
    return true;
}
