use nghe_proc_macro::api_derive;
use uuid::Uuid;

#[api_derive]
#[endpoint(path = "getCoverArt", url_only = true)]
#[derive(Clone, Copy)]
pub struct Request {
    pub id: Uuid,
    pub size: Option<u32>,
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    crate::test_roundtrip_request!(Request);
}
