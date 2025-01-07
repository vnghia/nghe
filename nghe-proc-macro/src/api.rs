use concat_string::concat_string;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Error, parse_quote, parse_str};

use crate::endpoint::Attribute;

#[derive(Debug, deluxe::ExtractAttributes)]
#[deluxe(attributes(endpoint))]
struct Endpoint {
    path: String,
    #[deluxe(flatten)]
    attribute: Attribute,
    #[deluxe(default = false)]
    url_only: bool,
    #[deluxe(default = true)]
    same_crate: bool,
}

#[derive(Debug, deluxe::ParseMetaItem)]
struct Derive {
    #[deluxe(default = true)]
    request: bool,
    #[deluxe(default = true)]
    response: bool,
    #[deluxe(default = true)]
    debug: bool,
    #[deluxe(default = true)]
    serde_apply: bool,
    #[deluxe(default = false)]
    serde_as: bool,
    #[deluxe(default = false)]
    fake: bool,
}

pub fn derive_endpoint(item: TokenStream) -> Result<TokenStream, Error> {
    let mut input: syn::ItemStruct = syn::parse2(item)?;
    let Endpoint { path, attribute, url_only, same_crate } =
        deluxe::extract_attributes(&mut input)?;

    let ident = &input.ident;
    if ident != "Request" {
        return Err(syn::Error::new(
            ident.span(),
            "Struct derived with `Endpoint` should be named `Request`",
        ));
    }

    let crate_path = if same_crate { format_ident!("crate") } else { format_ident!("nghe_api") };

    let impl_form = if attribute.form() {
        let url_form = concat_string!("/rest/", &path);
        let url_form_view = concat_string!("/rest/", &path, ".view");

        let mut auth_form_struct = input.clone();
        let auth_form_ident = format_ident!("AuthFormRequest");
        let mut auth_form_fields = None;

        auth_form_struct.attrs.clear();
        auth_form_struct.ident = auth_form_ident.clone();
        auth_form_struct.generics.params.push(parse_quote!('auth_u));
        auth_form_struct.generics.params.push(parse_quote!('auth_s));
        auth_form_struct.generics.params.push(parse_quote!('auth_p));
        auth_form_struct.fields = syn::Fields::Named(match input.fields {
            syn::Fields::Named(mut fields) => {
                auth_form_fields = Some(
                    fields
                        .named
                        .iter()
                        .map(|field| field.ident.as_ref().unwrap().clone())
                        .collect::<Vec<_>>(),
                );
                fields.named.push(parse_quote! {
                        #[serde(flatten, borrow)]
                        auth: #crate_path::auth::Form<'auth_u, 'auth_s, 'auth_p>
                });
                fields
            }
            syn::Fields::Unit => parse_quote! {{
                #[serde(flatten, borrow)]
                auth: #crate_path::auth::Form<'auth_u, 'auth_s, 'auth_p>
            }},
            syn::Fields::Unnamed(_) => {
                return Err(syn::Error::new(
                    ident.span(),
                    "Struct derived with `Endpoint` should be either named or unit struct",
                ));
            }
        });

        let impl_endpoint = if url_only {
            quote! {}
        } else {
            quote! {
                impl #crate_path::common::FormEndpoint for #ident {
                    type Response = Response;
                }
            }
        };

        let impl_auth_form_trait = if let Some(auth_form_fields) = auth_form_fields {
            quote! {
                fn new(request: #ident, auth: #crate_path::auth::Form<'u, 's, 'p>) -> Self {
                    let #ident { #(#auth_form_fields),* } = request;
                    Self { #(#auth_form_fields),*, auth }
                }

                fn request(self) -> #ident {
                    let Self { #(#auth_form_fields),*, auth } = self;
                    #ident { #(#auth_form_fields),* }
                }
            }
        } else {
            quote! {
                fn new(_: #ident, auth: #crate_path::auth::Form<'u, 's, 'p>) -> Self {
                    Self { auth }
                }

                fn request(self) -> #ident {
                    #ident
                }
            }
        };

        quote! {
            #[nghe_proc_macro::api_derive]
            #auth_form_struct

            impl #crate_path::common::FormURL for #ident {
                const URL_FORM: &'static str = #url_form;
                const URL_FORM_VIEW: &'static str = #url_form_view;
            }

            impl<'u, 's, 'p, 'de: 'u + 's + 'p>
            #crate_path::auth::form::Trait<'u, 's, 'p, 'de, #ident>
            for #auth_form_ident<'u, 's, 'p> {
                fn auth<'form>(&'form self) -> &'form #crate_path::auth::Form<'u, 's, 'p> {
                    &self.auth
                }

                #impl_auth_form_trait
            }

            impl<'u, 's, 'p, 'de: 'u + 's + 'p>
            #crate_path::common::FormRequest<'u, 's, 'p, 'de> for #ident {
                type AuthForm = #auth_form_ident<'u, 's, 'p>;
            }

            #impl_endpoint
        }
    } else {
        quote! {}
    };

    let impl_binary = if attribute.binary() {
        let url_binary = concat_string!("/rest/", &path, ".bin");

        let impl_endpoint = if url_only {
            quote! {}
        } else {
            quote! {
                impl #crate_path::common::BinaryEndpoint for #ident {
                    type Response = Response;
                }
            }
        };

        quote! {
            impl #crate_path::common::BinaryURL for #ident {
                const URL_BINARY: &'static str = #url_binary;
            }

            #impl_endpoint
        }
    } else {
        quote! {}
    };

    let impl_json = if attribute.json() {
        let url_json = concat_string!("/rest/", &path, ".json");

        let impl_endpoint = if url_only {
            quote! {}
        } else {
            quote! {
                impl #crate_path::common::JsonEndpoint for #ident {
                    type Response = Response;
                }
            }
        };

        quote! {
            impl #crate_path::common::JsonURL for #ident {
                const URL_JSON: &'static str = #url_json;
            }

            #impl_endpoint
        }
    } else {
        quote! {}
    };

    Ok(quote! {
        #impl_form
        #impl_binary
        #impl_json
    })
}

pub fn derive(args: TokenStream, item: TokenStream) -> Result<TokenStream, Error> {
    let args: Derive = deluxe::parse2(args)?;
    let input: syn::DeriveInput = syn::parse2(item)?;

    let ident = input.ident.to_string();
    let is_request_struct = ident == "Request";
    let has_serde = args.request || args.response;

    let is_enum = matches!(input.data, syn::Data::Enum(_));

    let mut derives: Vec<syn::Expr> = vec![];
    let mut attributes: Vec<syn::Attribute> = vec![];

    let endpoint_statement =
        if is_request_struct { Some(quote! {#[derive(nghe_proc_macro::Endpoint)]}) } else { None };

    if args.request {
        derives.push(parse_str("::serde::Deserialize")?);
    }
    if args.response {
        derives.push(parse_str("::serde::Serialize")?);
    }

    if args.debug {
        derives.push(parse_str("Debug")?);
    }

    if has_serde {
        if is_enum {
            attributes.push(parse_quote!(#[serde(rename_all_fields = "camelCase")]));
        }
        attributes.push(parse_quote!(#[serde(rename_all = "camelCase")]));
    };

    let apply_statement = if has_serde && args.serde_apply {
        quote! {
            #[serde_with::apply(
                Option => #[serde(skip_serializing_if = "Option::is_none", default)],
                Vec => #[serde(skip_serializing_if = "Vec::is_empty", default)],
                date::Date => #[serde(skip_serializing_if = "date::Date::is_none", default)],
                genre::Genres => #[serde(
                    skip_serializing_if = "genre::Genres::is_empty",
                    default
                )],
                OffsetDateTime => #[serde(with = "crate::time::serde")],
                Option<OffsetDateTime> => #[serde(with = "crate::time::serde::option")],
                time::Duration => #[serde(with = "crate::time::duration::serde")],
            )]
        }
    } else {
        quote! {}
    };
    let as_statement = if has_serde && args.serde_as {
        quote! { #[serde_with::serde_as] }
    } else {
        quote! {}
    };

    if args.fake {
        attributes
            .push(parse_quote!(#[cfg_attr(any(test, feature = "fake"), derive(fake::Dummy))]));
    }

    Ok(quote! {
        #endpoint_statement
        #apply_statement
        #as_statement
        #[derive(#(#derives),*)]
        #( #attributes )*
        #input
    })
}
