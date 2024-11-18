use concat_string::concat_string;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, parse_str, Error};

#[derive(Debug, deluxe::ParseMetaItem)]
struct CheckMusicFolder {
    #[deluxe(default = parse_quote!(request.music_folder_ids.as_ref()))]
    input: syn::Expr,
    #[deluxe(default = parse_quote!(with_user_id))]
    user_id: syn::Ident,
    #[deluxe(default = parse_quote!(with_music_folder))]
    music_folder: syn::Ident,
}

pub fn check_music_folder(args: TokenStream, item: TokenStream) -> Result<TokenStream, Error> {
    let check_user_id = item.to_string();
    let CheckMusicFolder { input, user_id, music_folder }: CheckMusicFolder = deluxe::parse2(args)?;

    let check_music_folder: syn::Expr = parse_str(&check_user_id.replace(
        &concat_string!(user_id.to_string(), "(user_id)"),
        &concat_string!(music_folder.to_string(), "(user_id, music_folder_ids)"),
    ))?;

    Ok(quote! {
        if let Some(music_folder_ids) = #input {
            #check_music_folder
        } else {
            #item
        }
    })
}
