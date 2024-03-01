use diesel::Queryable;
use serde::Serialize;
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
