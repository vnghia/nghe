pub mod create_playlist;

use nghe_proc_macro::api_derive;
use time::OffsetDateTime;
use uuid::Uuid;

#[api_derive]
pub struct Playlist {
    pub id: Uuid,
    pub name: String,
    pub comment: Option<String>,
    pub public: bool,
    pub song_count: u16,
    pub duration: u32,
    pub created: OffsetDateTime,
    pub changed: OffsetDateTime,
}
