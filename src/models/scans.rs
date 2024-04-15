use diesel::prelude::*;
pub use scans::*;
use uuid::Uuid;

pub use crate::schema::scans;

#[derive(Insertable)]
#[diesel(table_name = scans)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewScan {
    pub music_folder_id: Uuid,
}

#[derive(AsChangeset)]
#[diesel(table_name = scans)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ScanStat {
    pub scanned_song_count: i64,
    pub upserted_song_count: i64,
    pub deleted_song_count: i64,
    pub deleted_album_count: i64,
    pub deleted_artist_count: i64,
    pub deleted_genre_count: i64,
    pub scan_error_count: i64,
}
