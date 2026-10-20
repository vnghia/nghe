use nghe_proc_macro::api_derive;
use uuid::Uuid;

use super::Permission;

#[api_derive]
#[endpoint(path = "updatePermission")]
pub struct Request {
    pub user_id: Option<Uuid>,
    pub music_folder_id: Option<Uuid>,
    #[conf(flatten, prefix)]
    pub permission: Permission,
}

#[api_derive]
pub struct Response;

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    crate::test_roundtrip_json!(Request, Response);
}
