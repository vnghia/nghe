use nghe_proc_macro::api_derive;
use uuid::Uuid;

#[api_derive]
#[endpoint(path = "deleteUser", internal = true)]
pub struct Request {
    pub user_id: Uuid,
}

#[api_derive]
pub struct Response;
