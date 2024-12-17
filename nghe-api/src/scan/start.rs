use nghe_proc_macro::api_derive;
use uuid::Uuid;

#[api_derive]
#[derive(Clone, Copy, Default)]
pub struct Full {
    #[serde(default)]
    pub file: bool,
    #[serde(default)]
    pub dir_picture: bool,
}

#[api_derive]
#[endpoint(path = "startScan", internal = true)]
pub struct Request {
    pub music_folder_id: Uuid,
    #[serde(default)]
    pub full: Full,
}

#[api_derive]
pub struct Response;
