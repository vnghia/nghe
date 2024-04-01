use std::borrow::Cow;
use std::marker::ConstParamTy;

#[cfg(test)]
use derivative::Derivative;
use diesel::prelude::*;
use serde::Deserialize;
use time::OffsetDateTime;
pub use users::*;
use uuid::Uuid;

pub use crate::schema::users;

#[derive(
    Debug,
    Clone,
    Copy,
    ConstParamTy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Queryable,
    Selectable,
    Insertable,
    Deserialize,
)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[serde(rename_all = "camelCase")]
pub struct Role {
    pub admin_role: bool,
    pub stream_role: bool,
    pub download_role: bool,
    pub share_role: bool,
}

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
    #[diesel(embed)]
    pub role: Role,
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
    #[diesel(embed)]
    pub role: Role,
}

#[cfg(test)]
impl Role {
    pub const fn const_default() -> Self {
        Self { admin_role: false, stream_role: false, download_role: false, share_role: false }
    }
}

#[cfg(test)]
impl Default for Role {
    fn default() -> Self {
        Self::const_default()
    }
}
