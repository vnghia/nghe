use nghe_proc_macro::api_derive;

#[api_derive(endpoint = true)]
#[endpoint(path = "ping")]
pub struct Request {}

#[api_derive]
pub struct Response;