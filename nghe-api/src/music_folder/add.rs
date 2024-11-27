use nghe_proc_macro::api_derive;
use uuid::Uuid;

use crate::common::filesystem;

#[api_derive(fake = true)]
#[endpoint(path = "addMusicFolder")]
pub struct Request {
    pub name: String,
    pub path: String,
    pub ty: filesystem::Type,
    pub allow: bool,
}

#[api_derive]
pub struct Response {
    pub music_folder_id: Uuid,
}
