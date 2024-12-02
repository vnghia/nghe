use nghe_proc_macro::api_derive;

#[api_derive(fake = true)]
#[endpoint(path = "setupUser", internal = true)]
pub struct Request {
    pub username: String,
    pub password: String,
    pub email: String,
}

#[api_derive]
pub struct Response;
