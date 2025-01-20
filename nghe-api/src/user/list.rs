use nghe_proc_macro::api_derive;

use super::get;

#[api_derive(fake = true)]
#[endpoint(path = "listUser", internal = true)]
pub struct Request;

#[api_derive]
#[derive(Clone)]
pub struct Response {
    pub users: Vec<get::Response>,
}
