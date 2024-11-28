use nghe_proc_macro::api_derive;
use uuid::Uuid;

use crate::id3;

#[api_derive]
#[endpoint(path = "getPlayQueue")]
pub struct Request {}

#[api_derive]
#[derive(Default)]
pub struct Playqueue {
    pub entry: Vec<id3::song::Short>,
    pub current: Option<Uuid>,
    pub position: Option<u64>,
}

#[api_derive]
#[derive(Default)]
pub struct Response {
    #[serde(rename = "playQueue")]
    pub playqueue: Playqueue,
}
