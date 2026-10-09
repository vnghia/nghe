use nghe_proc_macro::api_derive;
use uuid::Uuid;

#[api_derive]
#[endpoint(path = "getMusicFolders")]
pub struct Request;

#[api_derive]
pub struct MusicFolder {
    pub id: Uuid,
    pub name: String,
}

#[api_derive]
pub struct MusicFolders {
    pub music_folder: Vec<MusicFolder>,
}

#[api_derive]
pub struct Response {
    pub music_folders: MusicFolders,
}
