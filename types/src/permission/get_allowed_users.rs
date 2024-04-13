use nghe_proc_macros::{add_common_convert, add_subsonic_response};
use uuid::Uuid;

#[add_common_convert]
pub struct GetAllowedUsersParams {
    pub id: Uuid,
}

#[add_subsonic_response]
pub struct GetAllowedUsersBody {
    pub ids: Vec<Uuid>,
}
