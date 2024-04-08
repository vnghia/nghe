use nghe_proc_macros::{add_common_convert, add_response_derive, add_subsonic_response};
use uuid::Uuid;

use crate::open_subsonic::common::id3::response::*;

#[add_common_convert]
#[derive(Debug)]
#[cfg_attr(feature = "test", derive(Default))]
pub struct Search3Params {
    pub artist_count: Option<i64>,
    pub artist_offset: Option<i64>,
    pub album_count: Option<i64>,
    pub album_offset: Option<i64>,
    pub song_count: Option<i64>,
    pub song_offset: Option<i64>,
    #[serde(rename = "musicFolderId")]
    pub music_folder_ids: Option<Vec<Uuid>>,
}

#[add_response_derive]
pub struct Search3Result {
    #[serde(rename = "artist", skip_serializing_if = "Vec::is_empty")]
    pub artists: Vec<ArtistId3>,
    #[serde(rename = "album", skip_serializing_if = "Vec::is_empty")]
    pub albums: Vec<AlbumId3>,
    #[serde(rename = "song", skip_serializing_if = "Vec::is_empty")]
    pub songs: Vec<SongId3>,
}

#[add_subsonic_response]
pub struct Search3Body {
    pub search_result3: Search3Result,
}
