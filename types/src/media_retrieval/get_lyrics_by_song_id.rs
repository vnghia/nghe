use nghe_proc_macros::{
    add_common_convert, add_request_types_test, add_subsonic_response, add_types_derive,
};
use uuid::Uuid;

use crate::id3::LyricId3;

#[add_common_convert]
#[derive(Debug)]
pub struct GetLyricsBySongIdParams {
    pub id: Uuid,
}

#[add_types_derive]
pub struct LyricList {
    pub structured_lyrics: Vec<LyricId3>,
}

#[add_subsonic_response]
pub struct GetLyricsBySongIdBody {
    pub lyrics_list: LyricList,
}

add_request_types_test!(GetLyricsBySongIdParams);
