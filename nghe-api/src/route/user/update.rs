use nghe_proc_macro::api_derive;
use uuid::Uuid;

#[api_derive]
#[endpoint(path = "updateUser")]
pub struct Request {
    pub id: Option<Uuid>,
    pub username: String,
    pub email: String,
}

#[api_derive]
pub struct Response;

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    crate::test_roundtrip!(Request, Response);
}
