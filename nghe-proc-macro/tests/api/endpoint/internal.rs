use nghe_proc_macro::api_derive;

#[api_derive]
#[endpoint(path = "path/endpoint", internal = true)]
pub struct Request {
    pub token: bool,
}
