use nghe_proc_macro::api_derive;

use super::get;

#[api_derive]
#[endpoint(path = "listUser")]
pub struct Request;

#[api_derive]
#[derive(Clone)]
pub struct Response {
    pub users: Vec<get::Response>,
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    crate::test_roundtrip!(Request, Response);
}
