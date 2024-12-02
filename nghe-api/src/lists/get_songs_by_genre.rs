use nghe_proc_macro::api_derive;
use uuid::Uuid;

use crate::id3;

#[api_derive]
#[endpoint(path = "getSongsByGenre")]
pub struct Request {
    pub genre: String,
    pub count: Option<u32>,
    pub offset: Option<u32>,
    #[serde(rename = "musicFolderId")]
    pub music_folder_ids: Option<Vec<Uuid>>,
}

#[api_derive]
pub struct SongsByGenre {
    pub song: Vec<id3::song::Full>,
}

#[api_derive]
pub struct Response {
    pub songs_by_genre: SongsByGenre,
}
