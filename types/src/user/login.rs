use nghe_proc_macros::{add_common_convert, add_request_types_test, add_subsonic_response};
use uuid::Uuid;

use super::Role;

#[add_common_convert]
pub struct LoginParams {}

#[add_subsonic_response]
pub struct LoginBody {
    pub id: Uuid,
    pub role: Role,
}

add_request_types_test!(LoginParams);
