use nghe_proc_macros::add_types_derive;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::id3::SongId3;

#[add_types_derive]
#[derive(Debug)]
pub struct PlaylistId3 {
    pub id: Uuid,
    pub name: String,
    pub public: bool,
    pub created: OffsetDateTime,
    pub changed: OffsetDateTime,
    pub song_count: u32,
    pub duration: u64,
}

#[add_types_derive]
#[derive(Debug)]
pub struct PlaylistId3WithSongs {
    #[serde(flatten)]
    pub playlist: PlaylistId3,
    #[serde(rename = "entry")]
    pub songs: Vec<SongId3>,
}
