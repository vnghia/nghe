use std::ops::Deref;
use std::vec::Vec;

use syn::parse::Parse;

#[derive(Debug)]
pub struct SynVec<T: Parse>(Vec<T>);

impl<T: Parse> Default for SynVec<T> {
    fn default() -> Self {
        vec![].into()
    }
}

impl<T: Parse> From<Vec<T>> for SynVec<T> {
    fn from(value: Vec<T>) -> Self {
        Self(value)
    }
}

impl<T: Parse> Deref for SynVec<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Parse> Parse for SynVec<T> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        syn::bracketed!(content in input);
        Ok(Self(content.parse_terminated(T::parse, syn::Token![,])?.into_iter().collect()))
    }
}

impl<T: Parse> darling::FromMeta for SynVec<T> {
    fn from_expr(expr: &syn::Expr) -> darling::Result<Self> {
        use quote::ToTokens;

        syn::parse2(expr.into_token_stream()).map_err(Into::into)
    }
}
