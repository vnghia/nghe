use nghe_proc_macro::api_derive;
use uuid::Uuid;

#[api_derive(endpoint = true)]
#[endpoint(path = "createUser")]
pub struct Request {
    pub username: String,
    pub password: String,
    pub email: String,

    pub admin: bool,
    pub stream: bool,
    pub download: bool,
    pub share: bool,

    pub allow: bool,
}

#[api_derive]
pub struct Response {
    pub user_id: Uuid,
}
