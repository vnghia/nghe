use derivative::Derivative;
use diesel::Queryable;
use serde::Serialize;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Queryable, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
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
    #[serde(with = "crate::utils::serde_format::time")]
    pub created: OffsetDateTime,
}
