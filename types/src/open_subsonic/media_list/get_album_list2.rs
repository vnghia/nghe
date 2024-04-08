use nghe_proc_macros::{
    add_common_convert, add_request_derive, add_response_derive, add_subsonic_response,
};
use uuid::Uuid;

use crate::open_subsonic::common::id3::response::*;

#[add_request_derive]
#[derive(Debug)]
pub enum GetAlbumListType {
    Random,
    Newest,
    Recent,
    ByYear,
    ByGenre,
    AlphabeticalByName,
}

#[add_common_convert]
#[derive(Debug)]
pub struct GetAlbumList2Params {
    #[serde(rename = "type")]
    pub list_type: GetAlbumListType,
    #[serde(rename = "size")]
    pub count: Option<i64>,
    pub offset: Option<i64>,
    #[serde(rename = "musicFolderId")]
    pub music_folder_ids: Option<Vec<Uuid>>,
    // By Year
    pub from_year: Option<i16>,
    pub to_year: Option<i16>,
    // By Genre
    pub genre: Option<String>,
}

#[add_response_derive]
#[derive(Debug)]
pub struct AlbumList2 {
    pub album: Vec<AlbumId3>,
}

#[add_subsonic_response]
pub struct GetAlbumList2Body {
    pub album_list2: AlbumList2,
}
