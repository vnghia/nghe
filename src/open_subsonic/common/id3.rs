use derivative::Derivative;
use diesel::{deserialize::FromSql, sql_types, Queryable};
use serde::Serialize;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Queryable, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DateId3 {
    pub year: Option<i16>,
    pub month: Option<i16>,
    pub day: Option<i16>,
}

#[derive(Debug, Queryable, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(Hash, Clone, PartialOrd, Ord))]
pub struct BasicArtistId3 {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Queryable, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArtistId3 {
    #[serde(flatten)]
    pub basic: BasicArtistId3,
}

#[derive(Derivative, Debug, Queryable, Serialize)]
#[derivative(PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BasicAlbumId3 {
    pub id: Uuid,
    pub name: String,
    pub song_count: i64,
    #[derivative(PartialEq = "ignore")]
    pub duration: f32,
    #[serde(with = "crate::utils::time::iso8601_datetime")]
    pub created: OffsetDateTime,
}

#[derive(Debug, Queryable, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AlbumId3 {
    #[serde(flatten)]
    pub basic: BasicAlbumId3,
    pub artists: Vec<BasicArtistId3>,
    pub year: Option<i16>,
    pub release_date: DateId3,
    pub original_release_date: DateId3,
}

#[derive(Derivative, Debug, Queryable, Serialize)]
#[derivative(PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BasicSongId3 {
    pub id: Uuid,
    pub title: String,
    #[derivative(PartialEq = "ignore")]
    pub duration: f32,
    pub size: i64,
    #[serde(with = "crate::utils::time::iso8601_datetime")]
    pub created: OffsetDateTime,
}

#[derive(Debug, Queryable, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SongId3 {
    #[serde(flatten)]
    pub basic: BasicSongId3,
    pub artists: Vec<BasicArtistId3>,
    #[serde(rename = "track")]
    pub track_number: Option<i32>,
    pub disc_number: Option<i32>,
}

pub type BasicArtistId3Record = sql_types::Record<(sql_types::Uuid, sql_types::Text)>;

impl FromSql<BasicArtistId3Record, diesel::pg::Pg> for BasicArtistId3 {
    fn from_sql(value: diesel::pg::PgValue<'_>) -> diesel::deserialize::Result<Self> {
        let (id, name) = FromSql::<BasicArtistId3Record, diesel::pg::Pg>::from_sql(value)?;
        Ok(BasicArtistId3 { id, name })
    }
}
