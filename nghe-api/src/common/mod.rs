use bitcode::{DecodeOwned, Encode};
use nghe_proc_macro::api_derive;
use serde::de::DeserializeOwned;
use serde::{Serialize, Serializer};
use serde_with::serde_as;

use super::constant;

#[serde_as]
#[api_derive(response = false)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct AuthToken(#[serde_as(as = "serde_with::hex::Hex")] [u8; 16]);

#[api_derive(response = false)]
#[derive(Clone, Copy)]
pub struct Auth<'u, 't> {
    #[serde(rename = "u")]
    pub username: &'u str,
    #[serde(rename = "s")]
    pub salt: &'t str,
    #[serde(rename = "t")]
    pub token: AuthToken,
}

#[api_derive(debug = false, bitcode = false)]
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

#[api_derive(debug = false, bitcode = false, response = true)]
pub struct SubsonicResponse<B> {
    #[serde(rename = "subsonic-response")]
    root: RootResponse<B>,
}

pub trait Endpoint: DeserializeOwned + Encode + DecodeOwned {
    const ENDPOINT: &'static str;
    const ENDPOINT_VIEW: &'static str;

    type Response: Serialize + Encode + DecodeOwned;
}

impl<'u, 't> Auth<'u, 't> {
    pub fn tokenize(password: impl AsRef<[u8]>, salt: impl AsRef<[u8]>) -> AuthToken {
        let password = password.as_ref();
        let salt = salt.as_ref();

        let mut data = Vec::with_capacity(password.len() + salt.len());
        data.extend_from_slice(password);
        data.extend_from_slice(salt);
        AuthToken(md5::compute(data).into())
    }

    pub fn check(password: impl AsRef<[u8]>, salt: impl AsRef<[u8]>, token: &AuthToken) -> bool {
        let password = password.as_ref();
        let salt = salt.as_ref();

        let password_token = Self::tokenize(password, salt);
        &password_token == token
    }
}

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
