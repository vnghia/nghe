use proc_macro::TokenStream;

mod api;

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

#[proc_macro_derive(Endpoint)]
pub fn derive_endpoint(_: TokenStream) -> TokenStream {
    api::derive_endpoint().into_token_stream()
}
