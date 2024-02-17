pub use crate::schema::configs;
pub use configs::*;

use diesel::prelude::*;
use std::borrow::Cow;
use time::OffsetDateTime;

#[derive(Debug, Identifiable, Queryable, Selectable, Clone, PartialEq, Eq)]
#[diesel(table_name = configs, primary_key(key))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Config {
    pub key: String,
    pub text: Option<String>,
    pub byte: Option<Vec<u8>>,
    pub updated_at: OffsetDateTime,
}

#[derive(Identifiable, Insertable)]
#[diesel(table_name = configs, primary_key(key))]
pub struct NewTextConfig<'a> {
    pub key: Cow<'a, str>,
    pub text: Cow<'a, str>,
}

#[derive(Identifiable, Insertable)]
#[diesel(table_name = configs, primary_key(key))]
pub struct NewByteConfig<'a> {
    pub key: Cow<'a, str>,
    pub byte: Cow<'a, [u8]>,
}
