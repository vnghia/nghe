use std::borrow::Cow;
use std::marker::ConstParamTy;

use diesel::prelude::*;
use nghe_proc_macros::add_convert_types;
use time::OffsetDateTime;
pub use users::*;
use uuid::Uuid;

pub use crate::schema::users;

#[add_convert_types(both = nghe_types::user::Role)]
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

#[add_convert_types(into = nghe_types::user::BasicUserId)]
#[derive(Debug, Queryable, Selectable, Insertable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[cfg_attr(test, derive(Clone))]
pub struct BasicUserId<'a> {
    pub id: Uuid,
    pub username: Cow<'a, str>,
}

#[add_convert_types(from = &'a nghe_types::user::BasicUser, refs(username))]
#[add_convert_types(into = nghe_types::user::BasicUser)]
#[derive(Debug, Queryable, Selectable, Insertable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[cfg_attr(test, derive(Clone))]
pub struct BasicUser<'a> {
    pub username: Cow<'a, str>,
    #[diesel(embed)]
    pub role: Role,
}

#[add_convert_types(into = nghe_types::user::User)]
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
