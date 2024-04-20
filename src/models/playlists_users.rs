use diesel::deserialize::{FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::pg::PgValue;
use diesel::prelude::*;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::Int2;
use diesel::{deserialize, serialize};
use nghe_proc_macros::add_convert_types;
pub use playlists_users::*;
use strum::FromRepr;
use uuid::Uuid;

pub use crate::schema::playlists_users;

#[repr(i16)]
#[derive(
    Debug, Clone, Copy, FromRepr, AsExpression, FromSqlRow, PartialEq, Eq, PartialOrd, Ord,
)]
#[diesel(sql_type = Int2)]
pub enum AccessLevel {
    Read = 1,
    Write = 2,
    Admin = 3,
}

#[add_convert_types(from = nghe_types::playlists::add_playlist_user::AddPlaylistUserParams)]
#[derive(Insertable)]
#[diesel(table_name = playlists_users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AddUser {
    pub playlist_id: Uuid,
    pub user_id: Uuid,
    pub access_level: AccessLevel,
}

impl ToSql<Int2, diesel::pg::Pg> for AccessLevel {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, diesel::pg::Pg>) -> serialize::Result {
        match *self {
            AccessLevel::Read => {
                <i16 as ToSql<Int2, diesel::pg::Pg>>::to_sql(&(AccessLevel::Read as i16), out)
            }
            AccessLevel::Write => {
                <i16 as ToSql<Int2, diesel::pg::Pg>>::to_sql(&(AccessLevel::Write as i16), out)
            }
            AccessLevel::Admin => {
                <i16 as ToSql<Int2, diesel::pg::Pg>>::to_sql(&(AccessLevel::Admin as i16), out)
            }
        }
    }
}

impl FromSql<Int2, diesel::pg::Pg> for AccessLevel {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        i16::from_sql(bytes)
            .map(|i| AccessLevel::from_repr(i).expect("database access level constraint violation"))
    }
}

impl From<AccessLevel> for nghe_types::playlists::access_level::AccessLevel {
    fn from(value: AccessLevel) -> Self {
        match value {
            AccessLevel::Read => Self::Read,
            AccessLevel::Write => Self::Write,
            AccessLevel::Admin => Self::Admin,
        }
    }
}

impl From<nghe_types::playlists::access_level::AccessLevel> for AccessLevel {
    fn from(value: nghe_types::playlists::access_level::AccessLevel) -> Self {
        match value {
            nghe_types::playlists::access_level::AccessLevel::Read => Self::Read,
            nghe_types::playlists::access_level::AccessLevel::Write => Self::Write,
            nghe_types::playlists::access_level::AccessLevel::Admin => Self::Admin,
        }
    }
}
