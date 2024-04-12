use nghe_proc_macros::{add_common_convert, add_subsonic_response};

#[add_common_convert]
pub struct DeleteUserParams {
    pub username: String,
}

#[add_subsonic_response]
pub struct DeleteUserBody {}
