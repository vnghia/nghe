use std::borrow::Cow;
use std::marker::ConstParamTy;

#[cfg(test)]
use derivative::Derivative;
use diesel::prelude::*;
use time::OffsetDateTime;
pub use users::*;
use uuid::Uuid;

pub use crate::schema::users;

#[derive(
    Debug, Queryable, Selectable, Insertable, ConstParamTy, PartialEq, Eq, PartialOrd, Ord,
)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
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

impl From<Role> for nghe_types::user::Role {
    fn from(value: Role) -> Self {
        Self {
            admin_role: value.admin_role,
            stream_role: value.stream_role,
            download_role: value.download_role,
            share_role: value.share_role,
        }
    }
}

impl From<nghe_types::user::Role> for Role {
    fn from(value: nghe_types::user::Role) -> Self {
        Self {
            admin_role: value.admin_role,
            stream_role: value.stream_role,
            download_role: value.download_role,
            share_role: value.share_role,
        }
    }
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
