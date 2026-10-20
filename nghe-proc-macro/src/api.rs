use concat_string::concat_string;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Error, parse_quote};

#[derive(Debug, darling::FromAttributes)]
#[darling(attributes(endpoint))]
struct Endpoint {
    path: String,
    #[darling(default = || false)]
    url_only: bool,
    #[darling(default = || format_ident!("crate"))]
    api_crate: syn::Ident,
}

#[derive(Debug, darling::FromMeta)]
struct SerdeApply {
    #[darling(default = || true)]
    enable: bool,
    #[darling(default = || true)]
    genre: bool,
    #[darling(default = || true)]
    date: bool,
    #[darling(default = || true)]
    time: bool,
}

#[derive(Debug, darling::FromMeta)]
struct TestDerive {
    #[darling(default = || true)]
    eq: bool,
    #[darling(default = || true)]
    bon: bool,
    #[darling(default = || true)]
    fake: bool,
}

#[derive(Debug, darling::FromMeta)]
#[darling(derive_syn_parse)]
struct Derive {
    #[darling(default = || true)]
    request: bool,
    #[darling(default = || true)]
    response: bool,
    #[darling(default = || None)]
    command: Option<bool>,
    #[darling(default = || true)]
    debug: bool,
    #[darling(default = || SerdeApply {
        enable: true, genre: true, date: true, time: true
    })]
    serde_apply: SerdeApply,
    #[darling(default = || false)]
    serde_as: bool,
    #[darling(default = || TestDerive {
        eq: true, bon: true, fake: true
    })]
    test: TestDerive,
}

pub fn derive_endpoint(item: TokenStream) -> Result<TokenStream, Error> {
    let input: syn::ItemStruct = syn::parse2(item)?;
    let Endpoint { path, url_only, api_crate } =
        darling::FromAttributes::from_attributes(&input.attrs)?;

    let ident = &input.ident;
    if ident != "Request" {
        return Err(syn::Error::new(
            ident.span(),
            "Struct derived with `Endpoint` should be named `Request`",
        ));
    }

    let url = concat_string!("/", &path);
    let url_view = concat_string!("/", &path, ".view");

    let mut auth_form_struct = input.clone();
    let auth_form_ident = format_ident!("AuthFormRequest");
    let mut auth_form_fields = None;

    let api_derive = {
        let serde_as = auth_form_struct
            .attrs
            .iter()
            .any(|attr| attr.path().segments.last().is_some_and(|path| path.ident == "serde_as"));
        syn::parse_quote!(#[nghe_proc_macro::api_derive(
            serde_as = #serde_as,
            test(bon = false),
        )])
    };

    auth_form_struct.attrs = vec![api_derive];
    auth_form_struct.ident = auth_form_ident.clone();
    auth_form_struct.generics.params.push(parse_quote!('auth_u));
    auth_form_struct.generics.params.push(parse_quote!('auth_c));
    auth_form_struct.generics.params.push(parse_quote!('auth_s));
    auth_form_struct.generics.params.push(parse_quote!('auth_p));

    let (fields, unit) = match input.fields {
        syn::Fields::Named(mut fields) => {
            auth_form_fields = Some(
                fields
                    .named
                    .iter()
                    .map(|field| field.ident.as_ref().unwrap().clone())
                    .collect::<Vec<_>>(),
            );
            fields.named.iter_mut().for_each(|field| {
                field.attrs.retain(|attr| !attr.path().is_ident("conf"));
            });
            fields.named.push(parse_quote! {
                    #[serde(flatten, borrow)]
                    auth: #api_crate::auth::Form<'auth_u, 'auth_c, 'auth_s, 'auth_p>
            });
            (fields, quote! {None})
        }
        syn::Fields::Unit => (
            parse_quote! {{
                #[serde(flatten, borrow)]
                auth: #api_crate::auth::Form<'auth_u, 'auth_c, 'auth_s, 'auth_p>
            }},
            quote! {
                Some(#ident)
            },
        ),
        syn::Fields::Unnamed(_) => {
            return Err(syn::Error::new(
                ident.span(),
                "Struct derived with `Endpoint` should be either named or unit struct",
            ));
        }
    };
    auth_form_struct.fields = syn::Fields::Named(fields);

    let impl_endpoint = if url_only {
        quote! {}
    } else {
        quote! {
            impl #api_crate::http::Endpoint for #ident {
                type Response = Response;
            }
        }
    };

    let impl_auth_form_trait = if let Some(auth_form_fields) = auth_form_fields {
        quote! {
            fn new(request: #ident, auth: #api_crate::auth::Form<'u, 'c, 's, 'p>) -> Self {
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
            fn new(_: #ident, auth: #api_crate::auth::Form<'u, 'c, 's, 'p>) -> Self {
                Self { auth }
            }

            fn request(self) -> #ident {
                #ident
            }
        }
    };

    Ok(quote! {
        #auth_form_struct

        impl #api_crate::http::Url for #ident {
            const URL: &'static str = #url;
            const URL_VIEW: &'static str = #url_view;
        }

        impl<'u, 'c, 's, 'p, 'de: 'u + 'c + 's + 'p>
        #api_crate::auth::form::Trait<'u, 'c, 's, 'p, 'de, #ident>
        for #auth_form_ident<'u, 'c, 's, 'p> {
            fn auth<'form>(&'form self) -> &'form #api_crate::auth::Form<'u, 'c, 's, 'p> {
                &self.auth
            }

            #impl_auth_form_trait
        }

        impl<'u, 'c, 's, 'p, 'de: 'u + 'c + 's + 'p>
        #api_crate::http::Request<'u, 'c, 's, 'p, 'de> for #ident {
            const UNIT: Option<Self> = #unit;

            type AuthForm = #auth_form_ident<'u, 'c, 's, 'p>;
        }

        #impl_endpoint
    })
}

fn derive_transform_attrs(
    cfg_attr_command: Option<&TokenStream>,
    cfg_attr_test: &TokenStream,
    original_attrs: &[syn::Attribute],
) -> Vec<syn::Attribute> {
    let mut conf_attr = None;
    let mut fake_attr = None;
    let mut attrs = vec![];

    for attr in original_attrs {
        let meta = &attr.meta;
        if let Some(ident) = meta.path().get_ident() {
            if ident == "conf" {
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

    if let Some(cfg_attr) = cfg_attr_command {
        let conf_attr = conf_attr.unwrap_or_else(|| parse_quote!(conf(long)));
        attrs.push(parse_quote!(#[cfg_attr(#cfg_attr, #conf_attr)]));
    }

    if let Some(ref fake_attr) = fake_attr {
        attrs.push(parse_quote!(#[cfg_attr(#cfg_attr_test, #fake_attr)]));
    }

    attrs
}

pub fn derive(args: TokenStream, item: TokenStream) -> Result<TokenStream, Error> {
    let args: Derive = syn::parse2(args)?;
    let mut input: syn::DeriveInput = syn::parse2(item)?;

    let ident = input.ident.to_string();
    let is_request_struct = ident == "Request";
    let has_serde = args.request || args.response;

    let cfg_attr_test = quote! {any(test, feature = "test")};
    let cfg_attr_command = if let Some(command) = args.command {
        if command { Some(quote! {feature = "command"}) } else { None }
    } else {
        if is_request_struct { Some(quote! {feature = "command"}) } else { None }
    };

    let input_derives = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("derive"))
        .map(|attr| {
            let mut derives = vec![];
            attr.parse_nested_meta(|meta| {
                derives.push(meta.path.clone());
                Ok(())
            })?;
            Ok::<_, Error>(derives)
        })
        .transpose()?
        .unwrap_or_default();

    let input_attributes = &input.attrs;

    let mut derives: Vec<syn::Path> = vec![];
    let mut test_derives: Vec<syn::Path> = vec![];
    let mut attributes: Vec<syn::Attribute> = vec![];
    let mut test_attributes: Vec<syn::Attribute> = vec![];

    let endpoint_statement =
        if is_request_struct { Some(quote! {#[derive(nghe_proc_macro::Endpoint)]}) } else { None };

    if args.request {
        derives.push(parse_quote! {::serde::Deserialize});
    }
    if args.response {
        derives.push(parse_quote! {::serde::Serialize});
    }

    if args.debug {
        derives.push(parse_quote! {Debug});
    }

    if args.test.eq {
        test_derives.push(parse_quote!(PartialEq));
    }
    if args.test.eq {
        test_derives.push(parse_quote!(Eq));
    }
    if args.test.fake {
        test_derives.push(parse_quote!(fake::Dummy));
    }

    if has_serde {
        attributes.push(parse_quote!(#[serde(rename_all = "camelCase")]));
    }

    let apply_statement = if has_serde && args.serde_apply.enable {
        let serde_apply = args.serde_apply;
        let mut applies = vec![
            quote! {Option => #[serde(skip_serializing_if = "Option::is_none", default)]},
            quote! {Vec => #[serde(skip_serializing_if = "Vec::is_empty", default)]},
        ];
        if serde_apply.genre {
            applies.push(quote! {
                genre::Genres => #[serde(skip_serializing_if = "genre::Genres::is_empty", default)]
            });
        }
        if serde_apply.date {
            applies.push(quote! {
                date::Date => #[serde(skip_serializing_if = "date::Date::is_none", default)]
            });
        }
        if serde_apply.time {
            applies.push(quote! {OffsetDateTime => #[serde(
                with = "crate::time::serde",
            )]});
            applies.push(quote! {Option<OffsetDateTime> => #[serde(
                with = "crate::time::serde::option"
            )]});
            applies.push(quote! {time::SignedDuration => #[serde(
                with = "crate::time::signed_duration::serde"
            )]});
        }
        quote! {
            #[serde_with::apply(
                #(#applies),*
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

    match input.data {
        syn::Data::Struct(ref mut data) => {
            if let syn::Fields::Named(ref mut fields) = data.fields {
                // Conf does not work with unit struct
                if let Some(ref cfg_attr) = cfg_attr_command {
                    attributes.push(parse_quote!(#[cfg_attr(#cfg_attr, derive(::conf::Conf))]));
                }
                for field in &mut fields.named {
                    field.attrs = derive_transform_attrs(
                        cfg_attr_command.as_ref(),
                        &cfg_attr_test,
                        &field.attrs,
                    );
                }

                if args.test.bon {
                    // Bon does not work with unit struct
                    test_derives.push(parse_quote!(bon::Builder));
                    test_attributes.push(parse_quote!(#[builder(
                        on(Vec<_>, default, into),
                        state_mod(vis = "pub"),
                    )]));
                }
            }
        }
        syn::Data::Enum(ref mut data) => {
            if has_serde {
                attributes.push(parse_quote!(#[serde(rename_all_fields = "camelCase")]));
            }
            if let Some(ref cfg_attr) = cfg_attr_command {
                attributes.push(parse_quote!(#[cfg_attr(#cfg_attr, derive(::conf::Subcommands))]));
            }

            for variant in &mut data.variants {
                variant.attrs = derive_transform_attrs(None, &cfg_attr_test, &variant.attrs);
                for field in &mut variant.fields {
                    field.attrs = derive_transform_attrs(
                        cfg_attr_command.as_ref(),
                        &cfg_attr_test,
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

    let test_derives: Vec<syn::Path> = test_derives
        .iter()
        .filter_map(|derive| {
            if input_derives.contains(&derive) { None } else { Some(parse_quote!(#derive)) }
        })
        .collect();

    let test_attributes: Vec<syn::Attribute> = test_attributes
        .iter()
        .filter_map(|attribute| {
            if input_attributes
                .iter()
                .find(|input_attribute| input_attribute.path() == attribute.path())
                .is_some()
            {
                None
            } else {
                let meta = &attribute.meta;
                Some(parse_quote!(#[cfg_attr(#cfg_attr_test, #meta)]))
            }
        })
        .collect();

    Ok(quote! {
        #endpoint_statement
        #apply_statement
        #as_statement
        #[derive(#(#derives),*)]
        #[cfg_attr(#cfg_attr_test, derive(#(#test_derives),*))]
        #( #attributes )*
        #( #test_attributes )*
        #input
    })
}
