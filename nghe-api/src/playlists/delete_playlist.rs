use nghe_proc_macro::api_derive;
use uuid::Uuid;

#[api_derive]
#[endpoint(path = "deletePlaylist")]
pub struct Request {
    pub id: Uuid,
}

#[api_derive]
pub struct Response;
