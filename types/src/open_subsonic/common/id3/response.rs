#![allow(clippy::too_many_arguments)]

use derive_new::new;
use isolang::Language;
use nghe_proc_macros::add_response_derive;
use time::OffsetDateTime;
use uuid::Uuid;

use super::super::id::MediaTypedId;

#[derive(Debug, Default)]
#[add_response_derive]
pub struct DateId3 {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub month: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub day: Option<u8>,
}

impl DateId3 {
    #[cfg(feature = "backend")]
    fn skip_serializing(&self) -> bool {
        self.year.is_none()
    }
}

#[derive(new, Debug)]
#[add_response_derive]
pub struct ArtistId3 {
    pub id: Uuid,
    pub name: String,
    #[new(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album_count: Option<u16>,
}

#[derive(new, Debug)]
#[add_response_derive]
pub struct AlbumId3 {
    pub id: Uuid,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<u16>,
    #[serde(skip_serializing_if = "DateId3::skip_serializing")]
    pub release_date: DateId3,
    #[serde(skip_serializing_if = "DateId3::skip_serializing")]
    pub original_release_date: DateId3,
    pub song_count: u16,
    pub duration: u32,
    #[serde(with = "crate::utils::time::iso8601_datetime")]
    pub created: OffsetDateTime,
    // Album cover art is dynamically computed based on allowed song cover arts.
    // So album covert art id is default to album id.
    pub cover_art: MediaTypedId,
    #[new(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub artists: Vec<ArtistId3>,
    #[new(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub genres: Vec<NameId3>,
}

#[derive(new, Debug)]
#[add_response_derive]
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
    #[new(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub genres: Vec<NameId3>,
}

#[derive(Debug)]
#[add_response_derive]
pub struct NameId3 {
    pub name: String,
}

#[derive(Debug)]
#[add_response_derive]
pub struct GenreId3 {
    pub value: String,
    pub song_count: u32,
    pub album_count: u32,
}

#[derive(Debug)]
#[add_response_derive]
pub struct LyricLineId3 {
    pub start: Option<u32>,
    pub value: String,
}

#[derive(Debug)]
#[add_response_derive]
pub struct LyricId3 {
    pub lang: Language,
    pub synced: bool,
    pub line: Vec<LyricLineId3>,
}

impl From<String> for NameId3 {
    fn from(name: String) -> Self {
        Self { name }
    }
}
