use proc_macro2::TokenStream;
use quote::quote;
use syn::fold::Fold;
use syn::{Error, parse_quote};

#[derive(Debug, deluxe::ParseMetaItem)]
struct CheckMusicFolder {
    #[deluxe(default = parse_quote!(request.music_folder_ids.as_ref()))]
    input: syn::Expr,
    #[deluxe(default = parse_quote!(with_user_id))]
    user_id: syn::Ident,
    #[deluxe(default = parse_quote!(with_music_folder))]
    music_folder: syn::Ident,
}

impl Fold for CheckMusicFolder {
    fn fold_expr_call(&mut self, expr: syn::ExprCall) -> syn::ExprCall {
        if let syn::Expr::Path(syn::ExprPath { path, .. }) = expr.func.as_ref()
            && let Some(segment) = path.segments.last()
            && segment.ident == self.user_id
            && expr.args.len() == 1
            && let Some(syn::Expr::Path(arg)) = expr.args.last()
            && let Some(arg) = arg.path.get_ident()
            && arg == "user_id"
        {
            let mut with_music_folder_path = path.clone();
            with_music_folder_path.segments.pop();
            with_music_folder_path.segments.push(syn::PathSegment {
                ident: self.music_folder.clone(),
                arguments: syn::PathArguments::None,
            });
            parse_quote! { #with_music_folder_path(user_id, music_folder_ids) }
        } else {
            expr
        }
    }
}

pub fn check_music_folder(args: TokenStream, item: TokenStream) -> Result<TokenStream, Error> {
    let check_user_id: syn::Expr = syn::parse2(item)?;
    let mut args: CheckMusicFolder = deluxe::parse2(args)?;
    let check_music_folder: syn::Expr = args.fold_expr(check_user_id.clone());

    let input = args.input;
    Ok(quote! {
        if let Some(music_folder_ids) = #input {
            #check_music_folder
        } else {
            #check_user_id
        }
    })
}
