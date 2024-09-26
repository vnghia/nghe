use std::borrow::Cow;

use diesel::prelude::*;
use diesel_derives::AsChangeset;

pub use crate::schema::genres::{self, *};

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = genres, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Upsert<'a> {
    pub value: Cow<'a, str>,
}

impl<'a> From<&'a str> for Upsert<'a> {
    fn from(genre: &'a str) -> Self {
        Self { value: genre.into() }
    }
}

impl<'a, 'b> From<&'a Cow<'b, str>> for Upsert<'b>
where
    'a: 'b,
{
    fn from(genre: &'a Cow<'b, str>) -> Self {
        Self { value: Cow::Borrowed(genre.as_ref()) }
    }
}
