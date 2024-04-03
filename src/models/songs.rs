use std::borrow::Cow;

use diesel::prelude::*;
pub use songs::*;
use uuid::Uuid;

pub use crate::schema::songs;

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
    pub format: Cow<'a, str>,
    pub duration: f32,
    pub bitrate: i32,
    pub sample_rate: i32,
    pub channel_count: i16,
    // Filesystem property
    pub file_hash: i64,
    pub file_size: i64,
    // Foreign key columns
    pub cover_art_id: Option<Uuid>,
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
    use crate::models::music_folders;
    use crate::open_subsonic::test::id3::db::*;

    #[derive(Queryable, Selectable)]
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
        #[diesel(select_expression = (
            songs::year,
            songs::month,
            songs::day,
        ))]
        #[diesel(select_expression_type = (
            songs::year,
            songs::month,
            songs::day,
        ))]
        pub date: DateId3Db,
        #[diesel(select_expression = (
            songs::release_year,
            songs::release_month,
            songs::release_day,
        ))]
        #[diesel(select_expression_type = (
            songs::release_year,
            songs::release_month,
            songs::release_day,
        ))]
        pub release_date: DateId3Db,
        #[diesel(select_expression = (
            songs::original_release_year,
            songs::original_release_month,
            songs::original_release_day,
        ))]
        #[diesel(select_expression_type = (
            songs::original_release_year,
            songs::original_release_month,
            songs::original_release_day,
        ))]
        pub original_release_date: DateId3Db,
        pub languages: Vec<Option<String>>,
        // Filesystem property
        #[diesel(embed)]
        pub music_folder: music_folders::MusicFolder,
        pub relative_path: String,
        pub file_hash: i64,
        pub file_size: i64,
        // Cover art
        pub cover_art_id: Option<Uuid>,
    }
}
