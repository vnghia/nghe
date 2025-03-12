use nghe_proc_macro::api_derive;
use uuid::Uuid;

use super::Permission;

#[api_derive]
#[endpoint(path = "updatePermission", internal = true)]
pub struct Request {
    pub user_id: Option<Uuid>,
    pub music_folder_id: Option<Uuid>,
    pub permission: Permission,
}

#[api_derive]
pub struct Response;
