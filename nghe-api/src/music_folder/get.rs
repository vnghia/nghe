use nghe_proc_macro::api_derive;
use uuid::Uuid;

use crate::common::filesystem;

#[api_derive(fake = true)]
#[endpoint(path = "getMusicFolder", internal = true)]
pub struct Request {
    pub id: Uuid,
}

#[api_derive]
#[derive(Clone)]
pub struct Response {
    pub name: String,
    pub path: String,
    #[serde(rename = "type")]
    pub ty: filesystem::Type,
    pub album_count: u64,
    pub song_count: u64,
}
