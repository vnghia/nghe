use nghe_proc_macro::api_derive;
use uuid::Uuid;

use super::Role;

#[api_derive(fake = true)]
#[endpoint(path = "updateUserRole", internal = true)]
pub struct Request {
    pub id: Uuid,
    pub role: Role,
}

#[api_derive]
pub struct Response;
