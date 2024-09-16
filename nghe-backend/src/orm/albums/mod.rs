use diesel::prelude::*;
use diesel_derives::AsChangeset;
use uuid::Uuid;

pub mod date;

use std::borrow::Cow;

use crate::schema::albums;

pub mod schema {
    pub use super::albums::*;
}

pub use schema::table;

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = albums, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Data<'a> {
    pub name: Cow<'a, str>,
    #[diesel(embed)]
    pub date: date::Date,
    #[diesel(embed)]
    pub release_date: date::Release,
    #[diesel(embed)]
    pub original_release_date: date::OriginalRelease,
    pub mbz_id: Option<Uuid>,
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = albums, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Upsert<'a> {
    pub music_folder_id: Uuid,
    #[diesel(embed)]
    pub data: Data<'a>,
}
