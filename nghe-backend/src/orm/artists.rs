use std::borrow::Cow;

use diesel::prelude::*;
use diesel_derives::AsChangeset;
use uuid::Uuid;

pub use crate::schema::artists::{self, *};

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = artists, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Data<'a> {
    pub name: Cow<'a, str>,
    pub mbz_id: Option<Uuid>,
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = artists, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Upsert<'a> {
    pub index: Cow<'a, str>,
    #[diesel(embed)]
    pub data: Data<'a>,
}
