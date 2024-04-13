use nghe_proc_macros::{add_common_convert, add_subsonic_response};

use super::common::BasicUserId;

#[add_common_convert]
pub struct GetBasicUserIdsParams {}

#[add_subsonic_response]
pub struct GetBasicUserIdsBody {
    pub basic_user_ids: Vec<BasicUserId>,
}
