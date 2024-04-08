use nghe_proc_macros::{add_request_derive, add_subsonic_response};
use serde_with::serde_as;

#[serde_as]
#[add_request_derive]
#[derive(Debug)]
#[cfg_attr(feature = "test", derive(fake::Dummy))]
pub struct SetupParams {
    pub username: String,
    #[serde_as(as = "serde_with::Bytes")]
    pub password: Vec<u8>,
    pub email: String,
}

#[add_subsonic_response]
#[derive(Debug)]
pub struct SetupBody {}
