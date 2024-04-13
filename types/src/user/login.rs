use nghe_proc_macros::{add_common_convert, add_subsonic_response};
use uuid::Uuid;

use super::Role;

#[add_common_convert]
pub struct LoginParams {}

#[add_subsonic_response]
pub struct LoginBody {
    pub id: Uuid,
    pub role: Role,
}
