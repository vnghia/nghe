use std::borrow::Cow;

use derivative::Derivative;
use nghe_proc_macros::add_types_derive;
use serde_with::serde_as;

pub type MD5Token = [u8; 16];

#[serde_as]
#[add_types_derive]
#[derive(Clone, Derivative, PartialEq, Eq)]
#[derivative(Debug)]
#[cfg_attr(feature = "test", derive(Default, fake::Dummy))]
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

pub fn to_password_token<P: AsRef<[u8]>, S: AsRef<[u8]>>(password: P, salt: S) -> MD5Token {
    let password = password.as_ref();
    let salt = salt.as_ref();

    let mut data = Vec::with_capacity(password.len() + salt.len());
    data.extend_from_slice(password);
    data.extend_from_slice(salt);
    md5::compute(data).into()
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

    fn with_common<T: Into<Cow<'common, CommonParams>>>(self, common: T) -> Self::Out;
}
