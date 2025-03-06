use nghe_proc_macro::api_derive;
use uuid::Uuid;

#[api_derive(fake = true)]
#[endpoint(path = "updateUserPassword", internal = true)]
pub struct Request {
    pub id: Option<Uuid>,
    pub password: String,
}

#[api_derive]
pub struct Response;
