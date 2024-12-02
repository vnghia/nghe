use nghe_proc_macro::api_derive;
use uuid::Uuid;

use super::Role;

#[api_derive(fake = true)]
#[endpoint(path = "createUser", internal = true)]
pub struct Request {
    pub username: String,
    pub password: String,
    pub email: String,
    pub role: Role,
    pub allow: bool,
}

#[api_derive]
pub struct Response {
    pub user_id: Uuid,
}
