use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::Error;
use crate::integration::lastfm;
use crate::integration::lastfm::model::artist;

#[serde_with::apply(
    Option => #[serde(skip_serializing_if = "Option::is_none")]
)]
#[derive(Debug, Serialize)]
struct Request<'a> {
    artist: Cow<'a, str>,
    limit: Option<u32>,
    page: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct ArtistMatches {
    artist: Vec<artist::Short>,
}

#[derive(Debug, Deserialize)]
struct Results {
    #[serde(rename = "artistmatches")]
    artist_matches: ArtistMatches,
}

#[derive(Debug, Deserialize)]
struct Response {
    results: Results,
}

impl lastfm::Request for Request<'_> {
    type Response = Response;
    const NAME: &'static str = "artist.search";
}

impl lastfm::Client {
    pub async fn artist_search(&self, artist: &str) -> Result<Option<artist::Short>, Error> {
        self.send(&Request { artist: artist.into(), limit: Some(1), page: None })
            .await
            .map(|response| response.results.artist_matches.artist.into_iter().next())
    }
}
