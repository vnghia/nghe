use nghe_proc_macro::api_derive;
use uuid::Uuid;

use crate::common::filesystem;

#[api_derive(fake = true)]
#[endpoint(path = "addMusicFolder", internal = true)]
pub struct Request {
    pub name: String,
    pub path: String,
    #[serde(rename = "type")]
    pub ty: filesystem::Type,
    pub allow: bool,
}

#[api_derive]
pub struct Response {
    pub music_folder_id: Uuid,
}
