use nghe_proc_macro::api_derive;
use uuid::Uuid;

use super::Song;
use crate::id3::genre;

#[api_derive]
pub struct Full {
    #[serde(flatten)]
    pub song: Song,
    pub album: String,
    pub album_id: Uuid,
    pub genres: genre::Genres,
}
