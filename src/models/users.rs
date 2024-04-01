use std::borrow::Cow;

#[cfg(test)]
use derivative::Derivative;
use diesel::prelude::*;
use time::OffsetDateTime;
pub use users::*;
use uuid::Uuid;

pub use crate::schema::users;

#[derive(Queryable, Selectable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[cfg_attr(test, derive(Derivative))]
#[cfg_attr(test, derivative(Default))]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password: Vec<u8>,
    pub email: String,
    pub admin_role: bool,
    pub download_role: bool,
    pub share_role: bool,
    #[cfg_attr(test, derivative(Default(value = "OffsetDateTime::UNIX_EPOCH")))]
    pub created_at: OffsetDateTime,
    #[cfg_attr(test, derivative(Default(value = "OffsetDateTime::UNIX_EPOCH")))]
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
