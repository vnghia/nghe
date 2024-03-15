pub use crate::schema::songs;
pub use songs::*;

use diesel::prelude::*;
use std::borrow::Cow;
use uuid::Uuid;

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = songs)]
#[diesel(treat_none_as_null = true)]
pub struct SongUpdateInformationDB<'a> {
    // Song tag
    pub title: Cow<'a, str>,
    pub album_id: Uuid,
    pub track_number: Option<i32>,
    pub track_total: Option<i32>,
    pub disc_number: Option<i32>,
    pub disc_total: Option<i32>,
    pub year: Option<i16>,
    pub month: Option<i16>,
    pub day: Option<i16>,
    pub release_year: Option<i16>,
    pub release_month: Option<i16>,
    pub release_day: Option<i16>,
    pub original_release_year: Option<i16>,
    pub original_release_month: Option<i16>,
    pub original_release_day: Option<i16>,
    pub languages: Vec<&'static str>,
    // Song property
    pub duration: f32,
    pub bitrate: i32,
    pub sample_rate: i32,
    pub channel_count: i16,
    // Filesystem property
    pub file_hash: i64,
    pub file_size: i64,
}

#[derive(Insertable)]
#[diesel(table_name = songs)]
pub struct SongFullInformationDB<'a> {
    #[diesel(embed)]
    pub update_information: SongUpdateInformationDB<'a>,
    // Filesystem property
    pub music_folder_id: Uuid,
    pub relative_path: Cow<'a, str>,
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[derive(Debug, Identifiable, Queryable, Selectable, Clone, PartialEq)]
    #[diesel(table_name = songs)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    pub struct Song {
        pub id: Uuid,
        // Song tag
        pub title: String,
        pub album_id: Uuid,
        pub track_number: Option<i32>,
        pub track_total: Option<i32>,
        pub disc_number: Option<i32>,
        pub disc_total: Option<i32>,
        pub year: Option<i16>,
        pub month: Option<i16>,
        pub day: Option<i16>,
        pub release_year: Option<i16>,
        pub release_month: Option<i16>,
        pub release_day: Option<i16>,
        pub original_release_year: Option<i16>,
        pub original_release_month: Option<i16>,
        pub original_release_day: Option<i16>,
        pub languages: Vec<Option<String>>,
        // Song property
        pub duration: f32,
        // Filesystem property
        pub music_folder_id: Uuid,
        pub relative_path: String,
        pub file_hash: i64,
        pub file_size: i64,
    }
}
