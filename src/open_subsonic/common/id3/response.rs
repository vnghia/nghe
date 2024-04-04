#![allow(clippy::too_many_arguments)]

use derive_new::new;
use serde::Serialize;
use time::OffsetDateTime;
use uuid::Uuid;

use super::super::id::MediaTypedId;

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

#[derive(new, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistId3 {
    pub id: Uuid,
    pub name: String,
    #[new(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album_count: Option<u16>,
}

#[derive(new, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumId3 {
    pub id: Uuid,
    pub name: String,
    pub song_count: u16,
    pub duration: u32,
    #[serde(with = "crate::utils::time::iso8601_datetime")]
    pub created: OffsetDateTime,
    // Album cover art is dynamically computed based on allowed song cover arts.
    // So album covert art id is default to album id.
    pub cover_art: MediaTypedId,
    #[new(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<u16>,
    #[new(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub artists: Vec<ArtistId3>,
    #[new(default)]
    #[serde(skip_serializing_if = "DateId3::skip_serializing")]
    pub release_date: DateId3,
    #[new(default)]
    #[serde(skip_serializing_if = "DateId3::skip_serializing")]
    pub original_release_date: DateId3,
}

#[derive(new, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SongId3 {
    pub id: Uuid,
    pub title: String,
    pub duration: u32,
    #[serde(with = "crate::utils::time::iso8601_datetime")]
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
    pub cover_art: Option<MediaTypedId>,
    #[new(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[new(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub artists: Vec<ArtistId3>,
}
