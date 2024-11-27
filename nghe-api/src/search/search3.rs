use nghe_proc_macro::api_derive;
use uuid::Uuid;

use crate::id3;

#[api_derive]
#[endpoint(path = "search3")]
pub struct Request {
    pub query: String,
    pub artist_count: Option<u32>,
    pub artist_offset: Option<u32>,
    pub album_count: Option<u32>,
    pub album_offset: Option<u32>,
    pub song_count: Option<u32>,
    pub song_offset: Option<u32>,
    #[serde(rename = "musicFolderId")]
    pub music_folder_ids: Option<Vec<Uuid>>,
}

#[serde_with::apply(
    Vec => #[serde(skip_serializing_if = "Vec::is_empty")],
)]
#[api_derive(response = true)]
pub struct SearchResult3 {
    pub artist: Vec<id3::artist::Artist>,
    pub album: Vec<id3::album::Album>,
    pub song: Vec<id3::song::Song>,
}

#[api_derive]
pub struct Response {
    pub search_result3: SearchResult3,
}
