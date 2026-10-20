use nghe_proc_macro::api_derive;
use uuid::Uuid;

#[api_derive]
#[endpoint(path = "getPlaylist")]
pub struct Request {
    pub id: Uuid,
}

#[api_derive]
pub struct Response {
    pub playlist: super::Full,
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    crate::test_roundtrip!(Request, Response);
}
