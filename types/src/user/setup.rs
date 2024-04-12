use nghe_proc_macros::{add_subsonic_response, add_types_derive};
use serde_with::serde_as;

#[serde_as]
#[add_types_derive]
#[derive(Debug)]
#[cfg_attr(feature = "test", derive(fake::Dummy))]
pub struct SetupParams {
    pub username: String,
    #[serde_as(as = "serde_with::hex::Hex")]
    pub password: Vec<u8>,
    pub email: String,
}

#[add_subsonic_response]
#[derive(Debug)]
pub struct SetupBody {}
