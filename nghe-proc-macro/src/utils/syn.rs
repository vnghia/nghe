use std::ops::Deref;
use std::vec::Vec as StdVec;

use syn::parse::Parse;

#[derive(Debug)]
pub struct Vec<T: Parse>(StdVec<T>);

impl<T: Parse> Default for Vec<T> {
    fn default() -> Self {
        vec![].into()
    }
}

impl<T: Parse> From<StdVec<T>> for Vec<T> {
    fn from(value: StdVec<T>) -> Self {
        Self(value)
    }
}

impl<T: Parse> Deref for Vec<T> {
    type Target = StdVec<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Parse> Parse for Vec<T> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        syn::bracketed!(content in input);
        Ok(Self(content.parse_terminated(T::parse, syn::Token![,])?.into_iter().collect()))
    }
}

impl<T: Parse> darling::FromMeta for Vec<T> {
    fn from_expr(expr: &syn::Expr) -> darling::Result<Self> {
        use quote::ToTokens;

        syn::parse2(expr.into_token_stream()).map_err(Into::into)
    }
}
