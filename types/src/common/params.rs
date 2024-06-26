use std::borrow::Cow;

use derivative::Derivative;
use nghe_proc_macros::add_types_derive;
use serde_with::serde_as;

pub type MD5Token = [u8; 16];

#[serde_as]
#[add_types_derive]
#[derive(Clone, Derivative, PartialEq, Eq)]
#[derivative(Debug)]
pub struct CommonParams {
    #[serde(rename = "u")]
    pub username: String,
    #[derivative(Debug = "ignore")]
    #[serde(rename = "s")]
    pub salt: String,
    #[derivative(Debug = "ignore")]
    #[serde(rename = "t")]
    #[serde_as(as = "serde_with::hex::Hex")]
    pub token: MD5Token,
}

pub fn to_password_token(password: impl AsRef<[u8]>, salt: impl AsRef<[u8]>) -> MD5Token {
    let password = password.as_ref();
    let salt = salt.as_ref();

    let mut data = Vec::with_capacity(password.len() + salt.len());
    data.extend_from_slice(password);
    data.extend_from_slice(salt);
    md5::compute(data).into()
}

impl AsRef<CommonParams> for &CommonParams {
    fn as_ref(&self) -> &CommonParams {
        self
    }
}

impl From<CommonParams> for Cow<'static, CommonParams> {
    fn from(value: CommonParams) -> Self {
        Cow::Owned(value)
    }
}

impl<'common> From<&'common CommonParams> for Cow<'common, CommonParams> {
    fn from(value: &'common CommonParams) -> Self {
        Cow::Borrowed(value)
    }
}

pub trait WithCommon<'common> {
    type Out;

    fn with_common(self, common: impl Into<Cow<'common, CommonParams>>) -> Self::Out;
}
