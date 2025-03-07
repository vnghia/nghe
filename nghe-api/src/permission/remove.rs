use nghe_proc_macro::api_derive;
use uuid::Uuid;

#[api_derive]
#[endpoint(path = "removePermission", internal = true)]
pub struct Request {
    pub user_id: Option<Uuid>,
    pub music_folder_id: Option<Uuid>,
}

#[api_derive]
pub struct Response;
