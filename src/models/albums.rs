pub use crate::schema::albums;
pub use albums::*;

use diesel::prelude::*;
use std::borrow::Cow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Identifiable, Queryable, Selectable, Clone, PartialEq, Eq)]
#[diesel(table_name = albums)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Album {
    pub id: Uuid,
    pub name: String,
    pub updated_at: OffsetDateTime,
    pub scanned_at: OffsetDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = albums)]
pub struct NewAlbum<'a> {
    pub name: Cow<'a, str>,
}
