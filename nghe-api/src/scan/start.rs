use nghe_proc_macro::api_derive;
use uuid::Uuid;

#[api_derive(endpoint = true)]
#[endpoint(path = "startScan")]
pub struct Request {
    pub music_folder_id: Uuid,
}

#[api_derive]
pub struct Response;