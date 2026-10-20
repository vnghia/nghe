use nghe_proc_macro::api_derive;
use uuid::Uuid;

use super::Role;

#[api_derive]
#[endpoint(path = "getUser")]
pub struct Request {
    pub id: Option<Uuid>,
}

#[api_derive]
#[derive(Clone)]
pub struct Response {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub role: Role,
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    crate::test_roundtrip!(Request, Response);
}
