#![deny(clippy::all)]
#![feature(let_chains)]
#![feature(try_blocks, yeet_expr)]
#![feature(proc_macro_span)]

mod db;
mod request;
mod response;
mod types;
mod utils;
use proc_macro::TokenStream;
use utils::*;

trait IntoTokenStream {
    fn into_token_stream(self) -> TokenStream;
}

impl IntoTokenStream for Result<proc_macro2::TokenStream, syn::Error> {
    fn into_token_stream(self) -> TokenStream {
        match self {
            Ok(r) => r.into(),
            Err(e) => e.to_compile_error().into(),
        }
    }
}

#[proc_macro_attribute]
pub fn add_subsonic_response(args: TokenStream, input: TokenStream) -> TokenStream {
    response::add_subsonic_response(args.into(), input.into()).into_token_stream()
}

#[proc_macro]
pub fn add_axum_response(item_ident: TokenStream) -> TokenStream {
    response::add_axum_response(item_ident.into()).into_token_stream()
}

#[proc_macro_attribute]
pub fn add_common_convert(_: TokenStream, input: TokenStream) -> TokenStream {
    request::add_common_convert(input.into()).into_token_stream()
}

#[proc_macro]
pub fn add_common_validate(input: TokenStream) -> TokenStream {
    request::add_common_validate(input.into()).into_token_stream()
}

#[proc_macro_attribute]
pub fn add_permission_filter(_: TokenStream, input: TokenStream) -> TokenStream {
    db::add_permission_filter(input.into()).into_token_stream()
}

#[proc_macro_attribute]
pub fn add_count_offset(args: TokenStream, input: TokenStream) -> TokenStream {
    db::add_count_offset(args.into(), input.into()).into_token_stream()
}

#[proc_macro]
pub fn generate_date_db(table_name: TokenStream) -> TokenStream {
    db::generate_date_db(table_name.into()).into_token_stream()
}

#[proc_macro_attribute]
pub fn add_types_derive(args: TokenStream, input: TokenStream) -> TokenStream {
    types::add_types_derive(args.into(), input.into()).into_token_stream()
}

#[proc_macro_attribute]
pub fn add_role_fields(_: TokenStream, input: TokenStream) -> TokenStream {
    types::add_role_fields(input.into()).into_token_stream()
}

#[proc_macro_attribute]
pub fn add_convert_types(args: TokenStream, input: TokenStream) -> TokenStream {
    types::add_convert_types(args.into(), input.into()).into_token_stream()
}

#[proc_macro]
pub fn add_request_types_test(params_type: TokenStream) -> TokenStream {
    types::add_request_types_test(params_type.into()).into_token_stream()
}
