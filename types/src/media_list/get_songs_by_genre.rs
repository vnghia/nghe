use nghe_proc_macros::{
    add_common_convert, add_request_types_test, add_subsonic_response, add_types_derive,
};
use uuid::Uuid;

use crate::id3::*;

#[add_common_convert]
#[derive(Debug)]
pub struct GetSongsByGenreParams {
    pub genre: String,
    pub count: Option<u32>,
    pub offset: Option<u32>,
    #[serde(rename = "musicFolderId")]
    pub music_folder_ids: Option<Vec<Uuid>>,
}

#[add_types_derive]
#[derive(Debug)]
pub struct SongsByGenre {
    pub song: Vec<SongId3>,
}

#[add_subsonic_response]
pub struct GetSongsByGenreBody {
    pub songs_by_genre: SongsByGenre,
}

add_request_types_test!(GetSongsByGenreParams);
