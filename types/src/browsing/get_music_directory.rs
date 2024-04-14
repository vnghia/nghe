use nghe_proc_macros::{
    add_common_convert, add_request_types_test, add_subsonic_response, add_types_derive,
};

use super::super::common::id::MediaTypedId;
use super::get_indexes::ChildItem;

#[add_common_convert]
#[derive(Debug)]
pub struct GetMusicDirectoryParams {
    pub id: MediaTypedId,
}

#[add_types_derive]
pub struct MusicDirectory {
    pub id: MediaTypedId,
    pub name: String,
    #[serde(rename = "child")]
    pub children: Vec<ChildItem>,
}

#[add_subsonic_response]
pub struct GetMusicDirectoryBody {
    pub directory: MusicDirectory,
}

add_request_types_test!(GetMusicDirectoryParams);
