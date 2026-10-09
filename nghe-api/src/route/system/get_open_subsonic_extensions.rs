use nghe_proc_macro::api_derive;

#[api_derive]
#[endpoint(path = "getOpenSubsonicExtensions")]
pub struct Request;

#[api_derive(request = false)]
pub struct Extension {
    pub name: &'static str,
    pub versions: &'static [u8],
}

#[api_derive(request = false)]
pub struct Response {
    pub open_subsonic_extensions: &'static [Extension],
}
