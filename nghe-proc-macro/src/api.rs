use concat_string::concat_string;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Error, parse_quote, parse_str};

#[derive(Debug, darling::FromAttributes)]
#[darling(attributes(endpoint))]
struct Endpoint {
    path: String,
    #[darling(default = || false)]
    url_only: bool,
    #[darling(default = || true)]
    same_crate: bool,
}

#[derive(Debug, darling::FromMeta)]
#[darling(derive_syn_parse)]
struct Derive {
    #[darling(default = || true)]
    request: bool,
    #[darling(default = || true)]
    response: bool,
    #[darling(default = || false)]
    command: bool,
    #[darling(default = || true)]
    debug: bool,
    #[darling(default = || true)]
    serde_apply: bool,
    #[darling(default = || false)]
    serde_as: bool,
    #[darling(default = || false)]
    fake: bool,
}

pub fn derive_endpoint(item: TokenStream) -> Result<TokenStream, Error> {
    let input: syn::ItemStruct = syn::parse2(item)?;
    let Endpoint { path, url_only, same_crate } =
        darling::FromAttributes::from_attributes(&input.attrs)?;

    let ident = &input.ident;
    if ident != "Request" {
        return Err(syn::Error::new(
            ident.span(),
            "Struct derived with `Endpoint` should be named `Request`",
        ));
    }

    let crate_path = if same_crate { format_ident!("crate") } else { format_ident!("nghe_api") };

    let url = concat_string!("/", &path);
    let url_view = concat_string!("/", &path, ".view");

    let mut auth_form_struct = input.clone();
    let auth_form_ident = format_ident!("AuthFormRequest");
    let mut auth_form_fields = None;

    auth_form_struct.attrs.clear();
    auth_form_struct.ident = auth_form_ident.clone();
    auth_form_struct.generics.params.push(parse_quote!('auth_u));
    auth_form_struct.generics.params.push(parse_quote!('auth_c));
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
                    auth: #crate_path::auth::Form<'auth_u, 'auth_c, 'auth_s, 'auth_p>
            });
            fields
        }
        syn::Fields::Unit => parse_quote! {{
            #[serde(flatten, borrow)]
            auth: #crate_path::auth::Form<'auth_u, 'auth_c, 'auth_s, 'auth_p>
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
            impl #crate_path::common::Endpoint for #ident {
                type Response = Response;
            }
        }
    };

    let impl_auth_form_trait = if let Some(auth_form_fields) = auth_form_fields {
        quote! {
            fn new(request: #ident, auth: #crate_path::auth::Form<'u, 'c, 's, 'p>) -> Self {
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
            fn new(_: #ident, auth: #crate_path::auth::Form<'u, 'c, 's, 'p>) -> Self {
                Self { auth }
            }

            fn request(self) -> #ident {
                #ident
            }
        }
    };

    Ok(quote! {
        #[nghe_proc_macro::api_derive]
        #auth_form_struct

        impl #crate_path::common::EndpointURL for #ident {
            const URL: &'static str = #url;
            const URL_VIEW: &'static str = #url_view;
        }

        impl<'u, 'c, 's, 'p, 'de: 'u + 'c + 's + 'p>
        #crate_path::auth::form::Trait<'u, 'c, 's, 'p, 'de, #ident>
        for #auth_form_ident<'u, 'c, 's, 'p> {
            fn auth<'form>(&'form self) -> &'form #crate_path::auth::Form<'u, 'c, 's, 'p> {
                &self.auth
            }

            #impl_auth_form_trait
        }

        impl<'u, 'c, 's, 'p, 'de: 'u + 'c + 's + 'p>
        #crate_path::common::Request<'u, 'c, 's, 'p, 'de> for #ident {
            type AuthForm = #auth_form_ident<'u, 'c, 's, 'p>;
        }

        #impl_endpoint
    })
}

fn derive_transform_attrs(
    cfg_attr_fake: Option<&TokenStream>,
    cfg_attr_command: Option<&TokenStream>,
    original_attrs: &[syn::Attribute],
) -> Vec<syn::Attribute> {
    let mut fake_attr = None;
    let mut conf_attr = None;
    let mut attrs = vec![];

    for attr in original_attrs {
        if let syn::Meta::List(meta) = &attr.meta
            && let Some(ident) = meta.path.get_ident()
        {
            if ident == "arg" || ident == "conf" {
                conf_attr = Some(meta.to_owned());
            } else if ident == "dummy" {
                fake_attr = Some(meta.to_owned());
            } else {
                attrs.push(attr.to_owned());
            }
        } else {
            attrs.push(attr.to_owned());
        }
    }

    if let Some(cfg_attr) = cfg_attr_fake
        && let Some(ref fake_attr) = fake_attr
    {
        attrs.push(parse_quote!(#[cfg_attr(#cfg_attr, #fake_attr)]));
    }

    if let Some(cfg_attr) = cfg_attr_command {
        let conf_attr = conf_attr.unwrap_or_else(|| parse_quote!(arg(long)));
        attrs.push(parse_quote!(#[cfg_attr(#cfg_attr, #conf_attr)]));
    }

    attrs
}

pub fn derive(args: TokenStream, item: TokenStream) -> Result<TokenStream, Error> {
    let args: Derive = syn::parse2(args)?;
    let mut input: syn::DeriveInput = syn::parse2(item)?;

    let ident = input.ident.to_string();
    let is_request_struct = ident == "Request";
    let has_serde = args.request || args.response;

    let cfg_attr_fake = if args.fake { Some(quote! {any(test, feature = "fake")}) } else { None };
    let cfg_attr_command =
        if is_request_struct || args.command { Some(quote! {feature = "command"}) } else { None };

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
        attributes.push(parse_quote!(#[serde(rename_all = "camelCase")]));
    }

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
                time::SignedDuration => #[serde(with = "crate::time::signed_duration::serde")],
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

    if let Some(ref cfg_attr) = cfg_attr_fake {
        attributes.push(parse_quote!(#[cfg_attr(#cfg_attr, derive(fake::Dummy))]));
    }

    match input.data {
        syn::Data::Struct(ref mut data) => {
            if let syn::Fields::Named(ref mut fields) = data.fields {
                // Conf does not work with unit struct
                if let Some(ref cfg_attr) = cfg_attr_command {
                    attributes.push(parse_quote!(#[cfg_attr(#cfg_attr, derive(::conf::Conf))]));
                }
                for field in &mut fields.named {
                    field.attrs = derive_transform_attrs(
                        cfg_attr_fake.as_ref(),
                        cfg_attr_command.as_ref(),
                        &field.attrs,
                    );
                }
            }
        }
        syn::Data::Enum(ref mut data) => {
            if has_serde {
                attributes.push(parse_quote!(#[serde(rename_all_fields = "camelCase")]));
            }

            for variant in &mut data.variants {
                variant.attrs = derive_transform_attrs(
                    cfg_attr_fake.as_ref(),
                    cfg_attr_command.as_ref(),
                    &variant.attrs,
                );
                for field in &mut variant.fields {
                    field.attrs = derive_transform_attrs(
                        cfg_attr_fake.as_ref(),
                        cfg_attr_command.as_ref(),
                        &field.attrs,
                    );
                }
            }
        }
        syn::Data::Union(_) => {
            if let Some(ref cfg_attr) = cfg_attr_command {
                attributes.push(parse_quote!(#[cfg_attr(#cfg_attr, derive(::conf::Conf))]));
            }
        }
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
