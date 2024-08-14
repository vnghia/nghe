use nghe_proc_macro::Endpoint;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Endpoint)]
#[endpoint(path = "setupUser")]
pub struct Request {
    pub username: String,
    pub password: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {}
