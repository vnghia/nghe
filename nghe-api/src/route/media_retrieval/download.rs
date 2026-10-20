use nghe_proc_macro::api_derive;
use uuid::Uuid;

#[api_derive]
#[endpoint(path = "download", url_only = true)]
pub struct Request {
    pub id: Uuid,
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    crate::test_roundtrip_request!(Request);
}
