use nghe_proc_macros::add_types_derive;
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

#[allow(dead_code)]
#[add_types_derive]
#[derive(Default)]
struct ConstantResponse {
    #[serde(serialize_with = "emit_open_subsonic_version", skip_deserializing)]
    version: (),
    #[serde(rename = "type", serialize_with = "emit_server_type", skip_deserializing)]
    server_type: (),
    #[serde(serialize_with = "emit_server_version", skip_deserializing)]
    server_version: (),
    #[serde(serialize_with = "emit_open_subsonic_support", skip_deserializing)]
    open_subsonic: (),
}

#[allow(dead_code)]
#[add_types_derive]
#[derive(Default)]
pub struct SuccessConstantResponse {
    #[serde(serialize_with = "emit_status_ok", skip_deserializing)]
    status: (),

    #[serde(flatten)]
    constant: ConstantResponse,
}

#[allow(dead_code)]
#[add_types_derive]
#[derive(Default)]
pub struct ErrorConstantResponse {
    #[serde(serialize_with = "emit_status_failed", skip_deserializing)]
    status: (),

    #[serde(flatten)]
    constant: ConstantResponse,
}

emit_constant_serialize!(open_subsonic_version, str, constant::OPEN_SUBSONIC_VERSION);
emit_constant_serialize!(server_type, str, constant::SERVER_NAME);
emit_constant_serialize!(server_version, str, constant::SERVER_VERSION);
emit_constant_serialize!(open_subsonic_support, bool, true);
emit_constant_serialize!(status_ok, str, "ok");
emit_constant_serialize!(status_failed, str, "failed");
