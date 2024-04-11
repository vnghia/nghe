use derivative::Derivative;
use nghe_proc_macros::{add_common_convert, add_subsonic_response};
use serde_with::serde_as;

use super::BasicUser;

#[serde_as]
#[add_common_convert]
#[derive(Derivative)]
#[derivative(Debug)]
pub struct CreateUserParams {
    pub basic: BasicUser,
    #[derivative(Debug = "ignore")]
    #[serde_as(as = "serde_with::Bytes")]
    pub password: Vec<u8>,
    pub email: String,
}

#[add_subsonic_response]
pub struct CreateUserBody {}
