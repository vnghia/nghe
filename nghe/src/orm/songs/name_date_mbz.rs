use std::borrow::Cow;

use diesel::prelude::*;
use uuid::Uuid;

use super::{date, songs};

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = songs, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct NameDateMbz<'a> {
    #[diesel(column_name = title)]
    pub name: Cow<'a, str>,
    #[diesel(embed)]
    pub date: date::Date,
    #[diesel(embed)]
    pub release_date: date::Release,
    #[diesel(embed)]
    pub original_release_date: date::OriginalRelease,
    pub mbz_id: Option<Uuid>,
}
