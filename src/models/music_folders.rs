use std::borrow::Cow;

use diesel::deserialize::{FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::pg::PgValue;
use diesel::prelude::*;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::Int2;
use diesel::{deserialize, serialize};
pub use music_folders::*;
use nghe_proc_macros::add_convert_types;
use strum::FromRepr;
use uuid::Uuid;

pub use crate::schema::music_folders;

#[derive(Debug, Clone, Copy, FromRepr, AsExpression, FromSqlRow, PartialEq, Eq)]
#[repr(i16)]
#[diesel(sql_type = Int2)]
#[cfg_attr(test, derive(fake::Dummy, strum::EnumIter, strum::AsRefStr, PartialOrd, Ord))]
pub enum FsType {
    Local = 1,
    S3 = 2,
}

#[add_convert_types(into = nghe_types::music_folder::MusicFolder, skips(path))]
#[add_convert_types(into = nghe_types::music_folder::MusicFolderPath)]
#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = music_folders)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[cfg_attr(test, derive(Clone, PartialEq, Eq, PartialOrd, Ord))]
pub struct MusicFolder {
    pub id: Uuid,
    pub name: String,
    pub path: String,
    pub fs_type: FsType,
}

#[derive(Insertable)]
#[diesel(table_name = music_folders)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewMusicFolder<'a> {
    pub path: Cow<'a, str>,
    pub name: Cow<'a, str>,
    pub fs_type: FsType,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = music_folders)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UpsertMusicFolder<'a> {
    pub name: Option<Cow<'a, str>>,
    pub path: Option<Cow<'a, str>>,
    pub fs_type: FsType,
}

impl ToSql<Int2, diesel::pg::Pg> for FsType {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, diesel::pg::Pg>) -> serialize::Result {
        match *self {
            FsType::Local => {
                <i16 as ToSql<Int2, diesel::pg::Pg>>::to_sql(&(FsType::Local as i16), out)
            }
            FsType::S3 => <i16 as ToSql<Int2, diesel::pg::Pg>>::to_sql(&(FsType::S3 as i16), out),
        }
    }
}

impl FromSql<Int2, diesel::pg::Pg> for FsType {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        i16::from_sql(bytes)
            .map(|i| FsType::from_repr(i).expect("database fs type constraint violation"))
    }
}

impl From<FsType> for nghe_types::music_folder::FsType {
    fn from(value: FsType) -> Self {
        match value {
            FsType::Local => Self::Local,
            FsType::S3 => Self::S3,
        }
    }
}

impl From<nghe_types::music_folder::FsType> for FsType {
    fn from(value: nghe_types::music_folder::FsType) -> Self {
        match value {
            nghe_types::music_folder::FsType::Local => Self::Local,
            nghe_types::music_folder::FsType::S3 => Self::S3,
        }
    }
}
