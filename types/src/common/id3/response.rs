#![allow(clippy::too_many_arguments)]

use derive_new::new;
#[cfg(test)]
use fake::Fake;
use isolang::Language;
use nghe_proc_macros::add_types_derive;
use time::OffsetDateTime;
use uuid::Uuid;

use super::super::id::MediaTypedId;
use super::super::time::time_serde;

#[add_types_derive]
#[derive(Debug, Default)]
pub struct DateId3 {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub year: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub month: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub day: Option<u8>,
}

impl DateId3 {
    fn skip_serializing(&self) -> bool {
        self.year.is_none()
    }
}

#[add_types_derive]
#[derive(new, Debug)]
pub struct ArtistId3 {
    pub id: Uuid,
    pub name: String,
    #[new(default)]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub album_count: Option<u16>,
    #[new(default)]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub music_brainz_id: Option<Uuid>,
}

#[add_types_derive]
#[derive(new, Debug)]
pub struct AlbumId3 {
    pub id: Uuid,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub year: Option<u16>,
    #[serde(skip_serializing_if = "DateId3::skip_serializing", default)]
    pub release_date: DateId3,
    #[serde(skip_serializing_if = "DateId3::skip_serializing", default)]
    pub original_release_date: DateId3,
    pub song_count: u16,
    pub duration: u32,
    #[serde(with = "time_serde::iso8601_datetime")]
    pub created: OffsetDateTime,
    // Album cover art is dynamically computed based on allowed song cover arts.
    // So album covert art id is default to album id.
    pub cover_art: MediaTypedId,
    #[new(default)]
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub artists: Vec<ArtistId3>,
    #[new(default)]
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub genres: Vec<NameId3>,
    #[new(default)]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub music_brainz_id: Option<Uuid>,
}

#[add_types_derive]
#[derive(new, Debug)]
pub struct SongId3 {
    pub id: Uuid,
    pub title: String,
    pub duration: u32,
    #[serde(with = "time_serde::iso8601_datetime")]
    pub created: OffsetDateTime,
    pub size: u32,
    pub suffix: String,
    pub bit_rate: u32,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub bit_depth: Option<u8>,
    pub sampling_rate: u32,
    pub channel_count: u8,
    pub album_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub year: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub track: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub disc_number: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub cover_art: Option<MediaTypedId>,
    #[new(default)]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub album: Option<String>,
    #[new(default)]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub content_type: Option<String>,
    #[new(default)]
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub artists: Vec<ArtistId3>,
    #[new(default)]
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub genres: Vec<NameId3>,
    #[new(default)]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub music_brainz_id: Option<Uuid>,
}

#[add_types_derive]
#[derive(Debug)]
pub struct NameId3 {
    pub name: String,
}

#[add_types_derive]
#[derive(Debug)]
pub struct GenreId3 {
    pub value: String,
    pub song_count: u32,
    pub album_count: u32,
}

#[add_types_derive]
#[derive(Debug)]
pub struct LyricLineId3 {
    pub start: Option<u32>,
    pub value: String,
}

#[add_types_derive]
#[derive(Debug)]
pub struct LyricId3 {
    #[cfg_attr(test, dummy(expr = "Language::from_usize((0..=7915).fake()).unwrap()"))]
    pub lang: Language,
    pub synced: bool,
    pub line: Vec<LyricLineId3>,
}

#[add_types_derive]
#[derive(Debug)]
pub struct InfoId3 {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub music_brainz_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub last_fm_url: Option<String>,
}

impl From<String> for NameId3 {
    fn from(name: String) -> Self {
        Self { name }
    }
}
