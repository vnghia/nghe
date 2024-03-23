use std::borrow::Cow;

pub use configs::*;
use diesel::prelude::*;

pub use crate::schema::configs;

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
