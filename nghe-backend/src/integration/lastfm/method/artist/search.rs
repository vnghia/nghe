use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::Error;
use crate::integration::lastfm;
use crate::integration::lastfm::model::artist;

#[serde_with::apply(
    Option => #[serde(skip_serializing_if = "Option::is_none")]
)]
#[derive(Debug, Serialize)]
pub struct Request<'a> {
    pub artist: Cow<'a, str>,
    pub limit: Option<u32>,
    pub page: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct ArtistMatches {
    pub artist: Vec<artist::Short>,
}

#[derive(Debug, Deserialize)]
pub struct Results {
    #[serde(rename = "artistmatches")]
    pub artist_matches: ArtistMatches,
}

#[derive(Debug, Deserialize)]
pub struct Response {
    pub results: Results,
}

impl lastfm::Request for Request<'_> {
    type Response = Response;
    const NAME: &'static str = "artist.search";
}

impl lastfm::Client {
    pub async fn artist_search(&self, artist: &str) -> Result<Response, Error> {
        self.send(&Request { artist: artist.into(), limit: Some(1), page: None }).await
    }
}
