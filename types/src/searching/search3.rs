use nghe_proc_macros::{
    add_common_convert, add_request_types_test, add_subsonic_response, add_types_derive,
};
use uuid::Uuid;

use crate::id3::*;

#[add_common_convert]
#[derive(Debug)]
#[cfg_attr(feature = "test", derive(Default))]
pub struct Search3Params {
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

#[add_types_derive]
pub struct Search3Result {
    #[serde(rename = "artist", skip_serializing_if = "Vec::is_empty", default)]
    pub artists: Vec<ArtistId3>,
    #[serde(rename = "album", skip_serializing_if = "Vec::is_empty", default)]
    pub albums: Vec<AlbumId3>,
    #[serde(rename = "song", skip_serializing_if = "Vec::is_empty", default)]
    pub songs: Vec<SongId3>,
}

#[add_subsonic_response]
pub struct Search3Body {
    pub search_result3: Search3Result,
}

add_request_types_test!(Search3Params);
