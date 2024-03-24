use derivative::Derivative;
use serde::Serialize;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DateId3 {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub month: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub day: Option<u8>,
}

impl DateId3 {
    fn skip_serializing(&self) -> bool {
        self.year.is_none()
    }
}

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ArtistId3 {
    pub id: Uuid,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album_count: Option<u16>,
}

#[derive(Debug, Serialize, Derivative)]
#[derivative(Default)]
#[serde(rename_all = "camelCase")]
pub struct AlbumId3 {
    pub id: Uuid,
    pub name: String,
    pub song_count: u16,
    pub duration: u32,
    #[serde(with = "crate::utils::time::iso8601_datetime")]
    #[derivative(Default(value = "OffsetDateTime::UNIX_EPOCH"))]
    pub created: OffsetDateTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<u16>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub artists: Vec<ArtistId3>,
    #[serde(skip_serializing_if = "DateId3::skip_serializing")]
    pub release_date: DateId3,
    #[serde(skip_serializing_if = "DateId3::skip_serializing")]
    pub original_release_date: DateId3,
}

#[derive(Debug, Serialize, Derivative)]
#[derivative(Default)]
#[serde(rename_all = "camelCase")]
pub struct SongId3 {
    pub id: Uuid,
    pub title: String,
    pub duration: u32,
    #[serde(with = "crate::utils::time::iso8601_datetime")]
    #[derivative(Default(value = "OffsetDateTime::UNIX_EPOCH"))]
    pub created: OffsetDateTime,
    pub size: u64,
    pub suffix: String,
    pub bit_rate: u32,
    pub album_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disc_number: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub artists: Vec<ArtistId3>,
}
