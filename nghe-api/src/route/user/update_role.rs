use nghe_proc_macro::api_derive;
use uuid::Uuid;

use super::Role;

#[api_derive]
#[endpoint(path = "updateUserRole")]
pub struct Request {
    pub id: Uuid,
    #[conf(flatten, prefix)]
    pub role: Role,
}

#[api_derive]
pub struct Response;
