use diesel::prelude::*;
use nghe_proc_macros::add_convert_types;
pub use scans::*;
use time::OffsetDateTime;
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

#[add_convert_types(into = nghe_types::scan::get_scan_status::ScanStatus)]
#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = scans)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ScanStatus {
    pub started_at: OffsetDateTime,
    pub finished_at: Option<OffsetDateTime>,
    pub unrecoverable: Option<bool>,
}
