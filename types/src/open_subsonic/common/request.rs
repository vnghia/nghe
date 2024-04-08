use derivative::Derivative;
use nghe_proc_macros::add_request_response_derive;
use serde_with::serde_as;

pub type MD5Token = [u8; 16];

#[serde_as]
#[add_request_response_derive]
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
