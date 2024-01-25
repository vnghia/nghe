pub use crate::schema::artists;
pub use artists::*;

use diesel::prelude::*;
use std::borrow::Cow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Identifiable, Queryable, Selectable, Clone, PartialEq, Eq)]
#[diesel(table_name = artists)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Artist {
    pub id: Uuid,
    pub name: String,
    pub index: String,
    pub updated_at: OffsetDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = artists)]
pub struct NewArtist<'a> {
    pub name: Cow<'a, str>,
    pub index: Cow<'a, str>,
}
