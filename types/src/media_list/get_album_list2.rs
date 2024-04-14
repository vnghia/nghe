use nghe_proc_macros::{
    add_common_convert, add_request_types_test, add_subsonic_response, add_types_derive,
};
use strum::EnumIter;
use uuid::Uuid;

use crate::id3::*;

#[add_types_derive]
#[derive(Debug, EnumIter)]
pub enum GetAlbumListType {
    Random,
    Newest,
    Frequent,
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
    pub count: Option<u32>,
    pub offset: Option<u32>,
    #[serde(rename = "musicFolderId")]
    pub music_folder_ids: Option<Vec<Uuid>>,
    // By Year
    pub from_year: Option<u16>,
    pub to_year: Option<u16>,
    // By Genre
    pub genre: Option<String>,
}

#[add_types_derive]
#[derive(Debug)]
pub struct AlbumList2 {
    pub album: Vec<AlbumId3>,
}

#[add_subsonic_response]
pub struct GetAlbumList2Body {
    pub album_list2: AlbumList2,
}

add_request_types_test!(GetAlbumList2Params);
