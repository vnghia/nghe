use nghe_proc_macro::api_derive;
use uuid::Uuid;

use super::Role;

#[api_derive(fake = true)]
#[endpoint(path = "getUser", internal = true)]
pub struct Request {
    pub id: Option<Uuid>,
}

#[api_derive]
#[derive(Clone)]
pub struct Response {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub role: Role,
}
