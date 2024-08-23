use nghe_proc_macro::api_derive;

#[api_derive(endpoint = true)]
#[endpoint(path = "createUser")]
pub struct Request {
    pub username: String,
    pub password: String,
    pub email: String,
    pub allow: bool,
}

#[api_derive]
pub struct Response {}
