pub use crate::schema::configs;
pub use configs::*;

use diesel::prelude::*;
use std::borrow::Cow;

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
