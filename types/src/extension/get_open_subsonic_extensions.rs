use nghe_proc_macros::{add_subsonic_response, add_types_derive};

#[add_types_derive]
#[derive(Debug)]
pub struct OSExtension {
    pub name: String,
    pub versions: Vec<u8>,
}

#[add_subsonic_response]
pub struct GetOpenSubsonicExtensionsBody {
    pub open_subsonic_extensions: Vec<OSExtension>,
}
