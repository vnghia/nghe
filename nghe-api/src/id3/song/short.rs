use nghe_proc_macro::api_derive;
use uuid::Uuid;

use super::Song;

#[api_derive]
pub struct Short {
    #[serde(flatten)]
    pub song: Song,
    pub album: String,
    pub album_id: Uuid,
}
