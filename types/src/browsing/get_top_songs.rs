use nghe_proc_macros::{add_common_convert, add_subsonic_response, add_types_derive};

use crate::id3::*;

#[add_common_convert]
#[derive(Debug)]
pub struct GetTopSongsParams {
    pub artist: String,
    pub count: Option<i64>,
}

#[add_types_derive]
#[derive(Debug)]
pub struct TopSongs {
    pub song: Vec<SongId3>,
}

#[add_subsonic_response]
pub struct GetTopSongsBody {
    pub top_songs: TopSongs,
}
