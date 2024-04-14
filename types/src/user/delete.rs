use nghe_proc_macros::{add_common_convert, add_request_types_test, add_subsonic_response};
use uuid::Uuid;

#[add_common_convert]
pub struct DeleteUserParams {
    pub id: Uuid,
}

#[add_subsonic_response]
pub struct DeleteUserBody {}

add_request_types_test!(DeleteUserParams);
