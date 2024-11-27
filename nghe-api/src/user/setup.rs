use nghe_proc_macro::api_derive;

#[api_derive]
#[endpoint(path = "setupUser")]
pub struct Request {
    pub username: String,
    pub password: String,
    pub email: String,
}

#[api_derive]
pub struct Response;
