use nghe_proc_macros::{
    add_common_convert, add_request_types_test, add_subsonic_response, add_types_derive,
};
use uuid::Uuid;

use crate::id3::*;

#[add_common_convert]
#[derive(Debug)]
pub struct GetRandomSongsParams {
    #[serde(rename = "size")]
    pub count: Option<u32>,
    #[serde(rename = "musicFolderId")]
    pub music_folder_ids: Option<Vec<Uuid>>,
    pub from_year: Option<u16>,
    pub to_year: Option<u16>,
    pub genre: Option<String>,
}

#[add_types_derive]
#[derive(Debug)]
pub struct RandomSongs {
    pub song: Vec<SongId3>,
}

#[add_subsonic_response]
pub struct GetRandomSongsBody {
    pub random_songs: RandomSongs,
}

add_request_types_test!(GetRandomSongsParams);
