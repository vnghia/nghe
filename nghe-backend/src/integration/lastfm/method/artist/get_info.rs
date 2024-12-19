use std::borrow::Cow;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Error;
use crate::integration::lastfm;
use crate::integration::lastfm::model::artist;

#[serde_with::apply(
    Option => #[serde(skip_serializing_if = "Option::is_none")]
)]
#[derive(Debug, Serialize)]
struct Request<'a> {
    artist: Option<Cow<'a, str>>,
    mbid: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
struct Response {
    artist: artist::Full,
}

impl lastfm::Request for Request<'_> {
    type Response = Response;
    const NAME: &'static str = "artist.getinfo";
}

impl lastfm::Client {
    pub async fn artist_get_info(
        &self,
        artist: &str,
        mbid: Option<Uuid>,
    ) -> Result<artist::Full, Error> {
        self.send(&Request {
            artist: if mbid.is_some() { Some(artist.into()) } else { None },
            mbid,
        })
        .await
        .map(|response| response.artist)
    }
}
