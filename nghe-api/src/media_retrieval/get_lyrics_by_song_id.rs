use nghe_proc_macro::api_derive;
use uuid::Uuid;

#[api_derive]
#[endpoint(path = "getLyricsBySongId")]
pub struct Request {
    pub id: Uuid,
}

#[api_derive]
pub struct Line {
    pub start: Option<u32>,
    pub value: String,
}

#[api_derive]
pub struct Lyric {
    pub display_title: Option<String>,
    pub lang: String,
    pub synced: bool,
    pub offset: u32,
    pub line: Vec<Line>,
}

#[api_derive]
pub struct LyricsList {
    pub structured_lyrics: Vec<Lyric>,
}

#[api_derive]
pub struct Response {
    pub lyrics_list: LyricsList,
}
