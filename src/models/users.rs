use std::borrow::Cow;
use std::marker::ConstParamTy;

use diesel::prelude::*;
pub use users::*;

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
