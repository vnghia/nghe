use nghe_proc_macro::api_derive;

use super::Role;

#[api_derive(fake = true)]
#[endpoint(path = "userInfo", internal = true)]
pub struct Request;

#[api_derive]
#[derive(Clone)]
pub struct Response {
    pub username: String,
    pub email: String,
    pub role: Role,
}
