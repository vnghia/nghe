use nghe_proc_macro::api_derive;
#[serde_with::apply(
    Option = >#[serde(skip_serializing_if = "Option::is_none", default)],
    Vec = >#[serde(skip_serializing_if = "Vec::is_empty", default)],
    genre::Genres = >#[serde(skip_serializing_if = "genre::Genres::is_empty", default)],
    date::Date = >#[serde(skip_serializing_if = "date::Date::is_none", default)],
    OffsetDateTime = >#[serde(serialize_with = "crate::time::serde::serialize")],
    Option<OffsetDateTime>= >#[serde(
        serialize_with = "crate::time::serde::option::serialize"
    )],
    time::SignedDuration = >#[serde(
        serialize_with = "crate::time::signed_duration::serde::serialize"
    )]
)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub token: bool,
}
#[automatically_derived]
impl ::core::fmt::Debug for Token {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field1_finish(
            f,
            "Token",
            "token",
            &&self.token,
        )
    }
}
