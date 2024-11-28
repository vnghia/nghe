use nghe_proc_macro::api_derive;
use uuid::Uuid;

#[api_derive]
#[endpoint(path = "savePlayQueue")]
pub struct Request {
    #[serde(rename = "id")]
    pub ids: Vec<Uuid>,
    pub current: Option<Uuid>,
    pub position: Option<u64>,
}

#[api_derive]
pub struct Response;
