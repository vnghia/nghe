use std::borrow::Cow;

use diesel::prelude::*;
use time::OffsetDateTime;
pub use users::*;
use uuid::Uuid;

pub use crate::schema::users;

#[derive(Debug, Identifiable, Queryable, Selectable, Clone, PartialEq, Eq)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password: Vec<u8>,
    pub email: String,
    pub admin_role: bool,
    pub download_role: bool,
    pub share_role: bool,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub username: Cow<'a, str>,
    pub password: Cow<'a, [u8]>,
    pub email: Cow<'a, str>,
    pub admin_role: bool,
    pub download_role: bool,
    pub share_role: bool,
}
