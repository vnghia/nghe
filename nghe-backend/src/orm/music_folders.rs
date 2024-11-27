use std::borrow::Cow;

use color_eyre::eyre::OptionExt;
use diesel::deserialize::{self, FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::pg::PgValue;
use diesel::prelude::*;
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Int2;
use nghe_api::music_folder::add::Request as AddRequest;
use o2o::o2o;
use strum::FromRepr;
use uuid::Uuid;

pub use crate::schema::music_folders::{self, *};

#[repr(i16)]
#[derive(Debug, Clone, Copy, FromRepr, AsExpression, FromSqlRow, PartialEq, Eq, o2o)]
#[diesel(sql_type = Int2)]
#[map_owned(nghe_api::common::filesystem::Type)]
pub enum FilesystemType {
    Local = 1,
    S3 = 2,
}

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = music_folders, check_for_backend(crate::orm::Type))]
pub struct Data<'a> {
    pub path: Cow<'a, str>,
    #[diesel(column_name = fs_type)]
    pub ty: FilesystemType,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = music_folders, check_for_backend(crate::orm::Type))]
pub struct MusicFolder<'a> {
    pub id: Uuid,
    #[diesel(embed)]
    pub data: Data<'a>,
}

#[derive(Debug, Insertable, AsChangeset, o2o)]
#[from_ref(AddRequest)]
#[diesel(table_name = music_folders, check_for_backend(crate::orm::Type))]
pub struct Upsert<'a> {
    #[from(AddRequest| Some((&~).into()))]
    pub name: Option<Cow<'a, str>>,
    #[from(AddRequest| Some((&~).into()))]
    pub path: Option<Cow<'a, str>>,
    #[from(AddRequest| Some(~.into()))]
    #[diesel(column_name = fs_type)]
    pub ty: Option<FilesystemType>,
}

impl ToSql<Int2, super::Type> for FilesystemType {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, super::Type>) -> serialize::Result {
        match self {
            FilesystemType::Local => {
                <i16 as ToSql<Int2, super::Type>>::to_sql(&(FilesystemType::Local as i16), out)
            }
            FilesystemType::S3 => {
                <i16 as ToSql<Int2, super::Type>>::to_sql(&(FilesystemType::S3 as i16), out)
            }
        }
    }
}

impl FromSql<Int2, super::Type> for FilesystemType {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        Ok(FilesystemType::from_repr(i16::from_sql(bytes)?)
            .ok_or_eyre("Database filesystem type constraint violation")?)
    }
}

impl Data<'_> {
    pub fn into_owned(self) -> Data<'static> {
        Data { path: self.path.into_owned().into(), ty: self.ty }
    }
}

impl MusicFolder<'_> {
    pub fn into_owned(self) -> MusicFolder<'static> {
        MusicFolder { id: self.id, data: self.data.into_owned() }
    }
}

mod query {
    use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
    use diesel_async::RunQueryDsl;
    use uuid::Uuid;

    use super::{music_folders, MusicFolder};
    use crate::database::Database;
    use crate::Error;

    impl MusicFolder<'static> {
        pub async fn query(database: &Database, id: Uuid) -> Result<Self, Error> {
            music_folders::table
                .filter(music_folders::id.eq(id))
                .select(Self::as_select())
                .get_result(&mut database.get().await?)
                .await
                .map_err(Error::from)
        }
    }
}
