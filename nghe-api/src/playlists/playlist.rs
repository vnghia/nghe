use bon::Builder;
use nghe_proc_macro::api_derive;
use time::OffsetDateTime;
use uuid::Uuid;

#[api_derive]
#[derive(Builder)]
#[builder(on(_, required))]
#[builder(state_mod(vis = "pub"))]
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

pub mod builder {
    pub use super::playlist_builder::*;
    pub use super::PlaylistBuilder as Builder;
}
