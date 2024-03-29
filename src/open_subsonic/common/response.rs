use serde::{Serialize, Serializer};

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

#[derive(Default, Serialize)]
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

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SuccessConstantResponse {
    #[serde(serialize_with = "emit_status_ok")]
    status: (),

    #[serde(flatten)]
    constant: ConstantResponse,
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
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

#[cfg(test)]
mod tests {
    use nghe_proc_macros::wrap_subsonic_response;
    use serde_json::{json, to_value};

    use super::constant;

    #[test]
    fn test_ser_success_empty() {
        #[wrap_subsonic_response]
        struct TestBody {}

        assert_eq!(
            to_value(Into::<SubsonicTestBody>::into(TestBody {})).unwrap(),
            json!({
                "subsonic-response": {
                    "status": "ok",
                    "version": constant::OPEN_SUBSONIC_VERSION,
                    "type": constant::SERVER_TYPE,
                    "serverVersion": constant::SERVER_VERSION,
                    "openSubsonic": true
                }
            })
        )
    }

    #[test]
    fn test_ser_success() {
        #[wrap_subsonic_response]
        struct TestBody {
            a: u16,
        }
        let a = 10;

        assert_eq!(
            to_value(Into::<SubsonicTestBody>::into(TestBody { a })).unwrap(),
            json!({
                "subsonic-response": {
                    "a": a,
                    "status": "ok",
                    "version": constant::OPEN_SUBSONIC_VERSION,
                    "type": constant::SERVER_TYPE,
                    "serverVersion": constant::SERVER_VERSION,
                    "openSubsonic": true
                }
            })
        )
    }

    #[test]
    fn test_ser_success_camel_case() {
        #[wrap_subsonic_response]
        struct TestBody {
            camel_case: u16,
        }
        let camel_case = 10;

        assert_eq!(
            to_value(Into::<SubsonicTestBody>::into(TestBody { camel_case })).unwrap(),
            json!({
                "subsonic-response": {
                    "camelCase": camel_case,
                    "status": "ok",
                    "version": constant::OPEN_SUBSONIC_VERSION,
                    "type": constant::SERVER_TYPE,
                    "serverVersion": constant::SERVER_VERSION,
                    "openSubsonic": true
                }
            })
        )
    }

    #[test]
    fn test_ser_error_empty() {
        #[wrap_subsonic_response(success = false)]
        struct TestBody {}

        assert_eq!(
            to_value(Into::<SubsonicTestBody>::into(TestBody {})).unwrap(),
            json!({
                "subsonic-response": {
                    "status": "failed",
                    "version": constant::OPEN_SUBSONIC_VERSION,
                    "type": constant::SERVER_TYPE,
                    "serverVersion": constant::SERVER_VERSION,
                    "openSubsonic": true
                }
            })
        )
    }

    #[test]
    fn test_ser_error() {
        #[wrap_subsonic_response(success = false)]
        struct TestBody {
            a: u16,
        }
        let a = 10;

        assert_eq!(
            to_value(Into::<SubsonicTestBody>::into(TestBody { a })).unwrap(),
            json!({
                "subsonic-response": {
                    "a": a,
                    "status": "failed",
                    "version": constant::OPEN_SUBSONIC_VERSION,
                    "type": constant::SERVER_TYPE,
                    "serverVersion": constant::SERVER_VERSION,
                    "openSubsonic": true
                }
            })
        )
    }
}
