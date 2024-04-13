use std::borrow::Cow;
use std::marker::ConstParamTy;

use diesel::prelude::*;
use time::OffsetDateTime;
pub use users::*;
use uuid::Uuid;

pub use crate::schema::users;

#[derive(
    Debug,
    Clone,
    Copy,
    Queryable,
    Selectable,
    Insertable,
    ConstParamTy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Role {
    pub admin_role: bool,
    pub stream_role: bool,
    pub download_role: bool,
    pub share_role: bool,
}

#[derive(Debug, Queryable, Selectable, Insertable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[cfg_attr(test, derive(Clone))]
pub struct BasicUser<'a> {
    pub username: Cow<'a, str>,
    #[diesel(embed)]
    pub role: Role,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: Uuid,
    #[diesel(embed)]
    pub basic: BasicUser<'static>,
    pub created_at: OffsetDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewUser<'a> {
    #[diesel(embed)]
    pub basic: BasicUser<'a>,
    pub password: Cow<'a, [u8]>,
    pub email: Cow<'a, str>,
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

impl<'a> From<BasicUser<'a>> for nghe_types::user::BasicUser {
    fn from(value: BasicUser<'a>) -> Self {
        Self { username: value.username.into(), role: value.role.into() }
    }
}

impl<'a> From<&'a nghe_types::user::BasicUser> for BasicUser<'a> {
    fn from(value: &'a nghe_types::user::BasicUser) -> Self {
        Self { username: (&value.username).into(), role: value.role.into() }
    }
}

impl From<User> for nghe_types::user::User {
    fn from(value: User) -> Self {
        Self { id: value.id, basic: value.basic.into(), created_at: value.created_at }
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
