pub mod filesystem;
pub mod format;

use bitcode::{DecodeOwned, Encode};
use nghe_proc_macro::api_derive;
use serde::de::DeserializeOwned;
use serde::{Serialize, Serializer};

use super::constant;

#[api_derive(debug = false, binary = false)]
struct RootResponse<B> {
    #[serde(serialize_with = "emit_open_subsonic_version")]
    version: (),
    #[serde(serialize_with = "emit_server_type")]
    r#type: (),
    #[serde(serialize_with = "emit_server_version")]
    server_version: (),
    #[serde(serialize_with = "emit_open_subsonic")]
    open_subsonic: (),
    #[serde(serialize_with = "emit_status_ok")]
    status: (),
    #[serde(flatten)]
    body: B,
}

#[api_derive(debug = false, binary = false)]
pub struct SubsonicResponse<B> {
    #[serde(rename = "subsonic-response")]
    root: RootResponse<B>,
}

pub trait Endpoint: DeserializeOwned + Encode + DecodeOwned {
    const ENDPOINT: &'static str;
    const ENDPOINT_VIEW: &'static str;
    const ENDPOINT_BINARY: &'static str;
}

pub trait EncodableEndpoint: Endpoint {
    type Response: Serialize + Encode + DecodeOwned;
}

pub trait BinaryEndpoint: Endpoint {}

impl<B> SubsonicResponse<B> {
    pub fn new(body: B) -> Self {
        Self {
            root: RootResponse {
                version: (),
                r#type: (),
                server_version: (),
                open_subsonic: (),
                status: (),
                body,
            },
        }
    }

    pub fn body(self) -> B {
        self.root.body
    }
}

macro_rules! emit_constant_serialize {
    ($constant_name:ident, $constant_type:ty, $constant_value:expr) => {
        paste::paste! {
            fn [<emit_ $constant_name>]<S: Serializer>(_: &(), s: S) -> Result<S::Ok, S::Error> {
                s.[<serialize_ $constant_type>]($constant_value)
            }
        }
    };
}

emit_constant_serialize!(open_subsonic_version, str, constant::OPEN_SUBSONIC_VERSION);
emit_constant_serialize!(server_type, str, constant::SERVER_NAME);
emit_constant_serialize!(server_version, str, constant::SERVER_VERSION);
emit_constant_serialize!(open_subsonic, bool, true);
emit_constant_serialize!(status_ok, str, "ok");

#[cfg(test)]
mod tests {
    use serde_json::{json, to_value};

    use super::*;

    #[test]
    fn test_serialize_empty() {
        #[api_derive(request = true, response = true, debug = false, binary = false)]
        struct TestBody {}

        assert_eq!(
            to_value(SubsonicResponse::new(TestBody {})).unwrap(),
            json!({
                "subsonic-response": {
                    "status": "ok",
                    "version": constant::OPEN_SUBSONIC_VERSION,
                    "type": constant::SERVER_NAME,
                    "serverVersion": constant::SERVER_VERSION,
                    "openSubsonic": true
                }
            })
        );
    }

    #[test]
    fn test_serialize() {
        #[api_derive(request = true, response = true, debug = false, binary = false)]
        struct TestBody {
            field: u16,
        }
        let field = 10;

        assert_eq!(
            to_value(SubsonicResponse::new(TestBody { field })).unwrap(),
            json!({
                "subsonic-response": {
                    "field": field,
                    "status": "ok",
                    "version": constant::OPEN_SUBSONIC_VERSION,
                    "type": constant::SERVER_NAME,
                    "serverVersion": constant::SERVER_VERSION,
                    "openSubsonic": true
                }
            })
        );
    }

    #[test]
    fn test_serialize_case() {
        #[api_derive(request = true, response = true, debug = false, binary = false)]
        struct TestBody {
            snake_case: u16,
        }
        let snake_case = 10;

        assert_eq!(
            to_value(SubsonicResponse::new(TestBody { snake_case })).unwrap(),
            json!({
                "subsonic-response": {
                    "snakeCase": snake_case,
                    "status": "ok",
                    "version": constant::OPEN_SUBSONIC_VERSION,
                    "type": constant::SERVER_NAME,
                    "serverVersion": constant::SERVER_VERSION,
                    "openSubsonic": true
                }
            })
        );
    }
}
