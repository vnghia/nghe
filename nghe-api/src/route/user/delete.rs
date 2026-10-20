use nghe_proc_macro::api_derive;
use uuid::Uuid;

#[api_derive]
#[endpoint(path = "deleteUser")]
#[derive(Clone, Copy)]
pub struct Request {
    pub user_id: Uuid,
}

#[api_derive]
pub struct Response;

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    crate::test_roundtrip!(Request, Response);
}
