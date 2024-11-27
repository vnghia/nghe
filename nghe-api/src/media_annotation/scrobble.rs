use nghe_proc_macro::api_derive;
use uuid::Uuid;

#[api_derive]
#[endpoint(path = "scrobble")]
pub struct Request {
    #[serde(rename = "id")]
    pub ids: Vec<Uuid>,
    #[serde(rename = "time")]
    pub times: Option<Vec<u64>>,
    pub submission: Option<bool>,
}

#[api_derive]
pub struct Response;
