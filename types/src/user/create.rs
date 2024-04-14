use derivative::Derivative;
use nghe_proc_macros::{
    add_common_convert, add_request_types_test, add_role_fields, add_subsonic_response,
};
use serde_with::serde_as;

// TODO: use [serde(flatten)] after https://github.com/serde-rs/serde/issues/1183
#[serde_as]
#[add_role_fields]
#[add_common_convert]
#[derive(Derivative)]
#[derivative(Debug)]
pub struct CreateUserParams {
    pub username: String,
    #[derivative(Debug = "ignore")]
    #[serde_as(as = "serde_with::hex::Hex")]
    pub password: Vec<u8>,
    pub email: String,
}

#[add_subsonic_response]
pub struct CreateUserBody {}

add_request_types_test!(CreateUserParams);
