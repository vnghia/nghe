use super::*;
use serde::{Serialize, Serializer};

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
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

fn emit_open_subsonic_version<S: Serializer>(_: &(), s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(constant::OPEN_SUBSONIC_VERSION)
}

fn emit_server_type<S: Serializer>(_: &(), s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(constant::SERVER_TYPE)
}

fn emit_server_version<S: Serializer>(_: &(), s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(constant::SERVER_VERSION)
}

fn emit_open_subsonic_support<S: Serializer>(_: &(), s: S) -> Result<S::Ok, S::Error> {
    s.serialize_bool(true)
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SuccessConstantResponse {
    #[serde(serialize_with = "emit_status_ok")]
    status: (),

    #[serde(flatten)]
    constant: ConstantResponse,
}

fn emit_status_ok<S: Serializer>(_: &(), s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str("ok")
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorConstantResponse {
    #[serde(serialize_with = "emit_status_failed")]
    status: (),

    #[serde(flatten)]
    constant: ConstantResponse,
}

fn emit_status_failed<S: Serializer>(_: &(), s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str("failed")
}

macro_rules! wrap_success_response_root {
    ($struct_name:ident, { $($field_name:ident : $field_type:ty),* }) => {
        paste::paste! {
          #[derive(Debug, Default, Serialize)]
          #[serde(rename_all = "camelCase")]
          struct [<Actual $struct_name>] {
            $($field_name : $field_type,)*
            #[serde(flatten)]
            constant: SuccessConstantResponse,
          }

          #[derive(Debug, Default, Serialize)]
          #[serde(rename_all = "camelCase")]
          pub struct $struct_name {
              #[serde(rename = "subsonic-response")]
              subsonic_response: [<Actual $struct_name>],
          }
        }
    };
}

pub(crate) use wrap_success_response_root;
