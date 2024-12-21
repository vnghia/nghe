use nghe_proc_macro::api_derive;
use uuid::Uuid;

use super::Album;
use crate::id3::{artist, song};

#[api_derive]
pub struct Full {
    #[serde(flatten)]
    pub album: Album,
    pub artist: String,
    pub artist_id: Uuid,
    pub artists: Vec<artist::Required>,
    pub is_compilation: bool,
    pub song: Vec<song::Short>,
}
