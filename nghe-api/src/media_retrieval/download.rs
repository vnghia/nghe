use nghe_proc_macro::api_derive;
use uuid::Uuid;

#[api_derive(endpoint = true)]
#[endpoint(path = "download", binary = true)]
pub struct Request {
    pub id: Uuid,
}
