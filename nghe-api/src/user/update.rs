use nghe_proc_macro::api_derive;
use uuid::Uuid;

#[api_derive(fake = true)]
#[endpoint(path = "updateUser", internal = true)]
pub struct Request {
    pub id: Option<Uuid>,
    pub username: String,
    pub email: String,
}

#[api_derive]
pub struct Response;
