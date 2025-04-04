use nghe_proc_macro::api_derive;

use crate::id3;

#[api_derive]
#[endpoint(path = "getTopSongs")]
pub struct Request {
    pub artist: String,
    pub count: Option<u32>,
}

#[api_derive]
pub struct TopSongs {
    pub song: Vec<id3::song::Full>,
}

#[api_derive]
pub struct Response {
    pub top_songs: TopSongs,
}
