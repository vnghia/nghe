use nghe_proc_macros::{add_common_convert, add_subsonic_response};

use super::User;

#[add_common_convert]
pub struct GetUsersParams {}

#[add_subsonic_response]
pub struct GetUsersBody {
    pub users: Vec<User>,
}
