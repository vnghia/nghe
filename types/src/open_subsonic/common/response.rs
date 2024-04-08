use nghe_proc_macros::add_response_derive;
#[cfg(not(feature = "frontend"))]
use serde::Serializer;

#[cfg(not(feature = "frontend"))]
use super::*;

macro_rules! emit_constant_serialize {
    ($constant_name:ident, $constant_type:ty, $constant_value:expr) => {
        paste::paste! {
            #[cfg(not(feature = "frontend"))]
            fn [<emit_ $constant_name>]<S: Serializer>(_: &(), s: S) -> Result<S::Ok, S::Error> {
                s.[<serialize_ $constant_type>]($constant_value)
            }
        }
    };
}

#[allow(dead_code)]
#[add_response_derive]
#[derive(Default)]
struct ConstantResponse {
    #[cfg_attr(not(feature = "frontend"), serde(serialize_with = "emit_open_subsonic_version"))]
    version: (),
    #[cfg_attr(
        not(feature = "frontend"),
        serde(rename = "type", serialize_with = "emit_server_type")
    )]
    server_type: (),
    #[cfg_attr(not(feature = "frontend"), serde(serialize_with = "emit_server_version"))]
    server_version: (),
    #[cfg_attr(not(feature = "frontend"), serde(serialize_with = "emit_open_subsonic_support"))]
    open_subsonic: (),
}

#[allow(dead_code)]
#[add_response_derive]
#[derive(Default)]
pub struct SuccessConstantResponse {
    #[cfg_attr(not(feature = "frontend"), serde(serialize_with = "emit_status_ok"))]
    status: (),

    #[serde(flatten)]
    constant: ConstantResponse,
}

#[allow(dead_code)]
#[add_response_derive]
#[derive(Default)]
pub struct ErrorConstantResponse {
    #[cfg_attr(not(feature = "frontend"), serde(serialize_with = "emit_status_failed"))]
    status: (),

    #[serde(flatten)]
    constant: ConstantResponse,
}

emit_constant_serialize!(open_subsonic_version, str, constant::OPEN_SUBSONIC_VERSION);
emit_constant_serialize!(server_type, str, constant::SERVER_TYPE);
emit_constant_serialize!(server_version, str, constant::SERVER_VERSION);
emit_constant_serialize!(open_subsonic_support, bool, true);
emit_constant_serialize!(status_ok, str, "ok");
emit_constant_serialize!(status_failed, str, "failed");
