use std::borrow::Cow;

use diesel::prelude::*;
use diesel_derives::AsChangeset;

pub use crate::schema::configs::{self, *};

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = configs, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Data<'a> {
    pub text: Option<Cow<'a, str>>,
    pub byte: Option<Cow<'a, [u8]>>,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = configs, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Upsert<'a> {
    pub key: &'static str,
    #[diesel(embed)]
    pub data: Data<'a>,
}
