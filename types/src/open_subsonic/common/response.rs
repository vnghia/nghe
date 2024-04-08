use nghe_proc_macros::add_response_derive;
use serde::Serializer;

use super::*;

macro_rules! emit_constant_serialize {
    ($constant_name:ident, $constant_type:ty, $constant_value:expr) => {
        paste::paste! {
          fn [<emit_ $constant_name>]<S: Serializer>(_: &(), s: S) -> Result<S::Ok, S::Error> {
            s.[<serialize_ $constant_type>]($constant_value)
          }
        }
    };
}

#[add_response_derive]
#[derive(Default)]
struct ConstantResponse {
    #[serde(serialize_with = "emit_open_subsonic_version")]
    version: (),
    #[serde(rename = "type", serialize_with = "emit_server_type")]
    server_type: (),
    #[serde(serialize_with = "emit_server_version")]
    server_version: (),
    #[serde(serialize_with = "emit_open_subsonic_support")]
    open_subsonic: (),
}

#[add_response_derive]
#[derive(Default)]
pub struct SuccessConstantResponse {
    #[serde(serialize_with = "emit_status_ok")]
    status: (),

    #[serde(flatten)]
    constant: ConstantResponse,
}

#[add_response_derive]
#[derive(Default)]
pub struct ErrorConstantResponse {
    #[serde(serialize_with = "emit_status_failed")]
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
