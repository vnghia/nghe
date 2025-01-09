use nghe_proc_macro::api_derive;

use crate::auth::ApiKey;

#[api_derive]
#[endpoint(path = "createKey", internal = true)]
pub struct Request;

#[api_derive]
#[serde(transparent)]
pub struct Response {
    pub api_key: ApiKey,
}
