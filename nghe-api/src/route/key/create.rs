use nghe_proc_macro::api_derive;

use crate::auth::ApiKey;

#[api_derive]
#[derive(Clone)]
#[endpoint(path = "createKey")]
pub struct Request;

#[api_derive]
#[serde(transparent)]
pub struct Response {
    pub api_key: ApiKey,
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    crate::test_roundtrip!(Request, Response);
}
