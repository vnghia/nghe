use nghe_proc_macros::{add_common_convert, add_response_derive, add_subsonic_response};
use uuid::Uuid;

use super::super::common::id3::response::LyricId3;

#[add_common_convert]
#[derive(Debug)]
pub struct GetLyricsBySongIdParams {
    pub id: Uuid,
}

#[add_response_derive]
pub struct LyricList {
    pub structured_lyrics: Vec<LyricId3>,
}

#[add_subsonic_response]
pub struct GetLyricsBySongIdBody {
    pub lyrics_list: LyricList,
}
