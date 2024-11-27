use nghe_proc_macro::api_derive;
use uuid::Uuid;

use crate::id3;

#[api_derive]
#[endpoint(path = "getRandomSongs")]
pub struct Request {
    pub size: Option<u32>,
    pub genre: Option<String>,
    pub from_year: Option<u16>,
    pub to_year: Option<u16>,
    #[serde(rename = "musicFolderId")]
    pub music_folder_ids: Option<Vec<Uuid>>,
}

#[api_derive(response = true)]
pub struct RandomSong {
    pub song: Vec<id3::song::Full>,
}

#[api_derive]
pub struct Response {
    pub random_songs: RandomSong,
}
