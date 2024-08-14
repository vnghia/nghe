use nghe_proc_macro::Endpoint;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Endpoint)]
pub struct Request {
    pub username: String,
    pub password: String,
    pub email: String,
    pub allow: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {}
