#![deny(clippy::all)]
#![feature(proc_macro_span)]

mod params;

use proc_macro::TokenStream;

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

#[proc_macro_derive(MethodName)]
pub fn add_method_name(input: TokenStream) -> TokenStream {
    params::add_method_name(input.into()).into_token_stream()
}
