use nghe_proc_macro::api_derive;

use crate::auth::ApiKey;

#[api_derive]
#[derive(Clone)]
#[endpoint(path = "createKey", internal = true)]
pub struct Request {
    pub username: String,
    pub password: String,
    pub client: String,
}

#[api_derive]
#[serde(transparent)]
pub struct Response {
    pub api_key: ApiKey,
}
