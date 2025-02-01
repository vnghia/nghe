use nghe_proc_macro::api_derive;

#[api_derive]
#[endpoint(path = "health")]
pub struct Request;

#[api_derive(request = false)]
pub struct Response;
