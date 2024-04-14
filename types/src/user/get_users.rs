use nghe_proc_macros::{add_common_convert, add_request_types_test, add_subsonic_response};

use super::User;

#[add_common_convert]
pub struct GetUsersParams {}

#[add_subsonic_response]
pub struct GetUsersBody {
    pub users: Vec<User>,
}

add_request_types_test!(GetUsersParams);
