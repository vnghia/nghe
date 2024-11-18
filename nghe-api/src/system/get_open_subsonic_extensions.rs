use nghe_proc_macro::api_derive;

#[api_derive(endpoint = true, binary = false)]
#[endpoint(path = "getOpenSubsonicExtensions")]
#[endpoint(binary = false)]
pub struct Request {}

#[api_derive(response = true, binary = false)]
pub struct Extension {
    pub name: &'static str,
    pub versions: &'static [u8],
}

#[api_derive(binary = false)]
pub struct Response {
    pub open_subsonic_extensions: &'static [Extension],
}
