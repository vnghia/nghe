use nghe_proc_macro::api_derive;
use uuid::Uuid;

#[api_derive]
#[endpoint(path = "updateArtistInformation")]
pub struct Request {
    pub artist_id: Uuid,
    pub spotify_id: Option<String>,
    pub lastfm_name: Option<String>,
}

#[api_derive]
pub struct Response;

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    crate::test_roundtrip!(Request, Response);
}
