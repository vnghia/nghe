use nghe_proc_macros::{add_response_derive, add_subsonic_response};

#[add_response_derive]
#[derive(Debug)]
pub struct OSExtension {
    pub name: String,
    pub versions: Vec<u8>,
}

#[add_subsonic_response]
pub struct GetOpenSubsonicExtensionsBody {
    pub open_subsonic_extensions: Vec<OSExtension>,
}
