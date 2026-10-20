use nghe_proc_macro::api_derive;

#[api_derive]
#[derive(Clone)]
#[endpoint(path = "setupUser")]
pub struct Request {
    pub username: String,
    pub password: String,
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
