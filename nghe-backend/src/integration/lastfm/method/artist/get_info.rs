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
    pub async fn fetch_artist(
        &self,
        artist: &str,
        mbid: Option<Uuid>,
    ) -> Result<artist::Full, Error> {
        self.send(&Request {
            artist: if mbid.is_none() { Some(artist.into()) } else { None },
            mbid,
        })
        .await
        .map(|response| response.artist)
    }
}

#[cfg(all(test, lastfm_env))]
#[coverage(off)]
mod tests {
    use rstest::rstest;
    use uuid::uuid;

    use super::*;
    use crate::config;

    #[rstest]
    #[case(
        "Cher",
        Some(uuid!("bfcc6d75-a6a5-4bc6-8282-47aec8531818")),
        "https://www.last.fm/music/Cher",
    )]
    #[case("non-existent girl", None, "https://www.last.fm/music/non-existent+girl")]
    #[tokio::test]
    async fn test_fetch_artist(#[case] name: &str, #[case] mbid: Option<Uuid>, #[case] url: &str) {
        let client = lastfm::Client::new(
            reqwest::Client::default(),
            config::integration::Lastfm::from_env(),
        )
        .unwrap();
        let artist = client.fetch_artist(name, mbid).await.unwrap();
        assert_eq!(artist.short.name, name);
        assert_eq!(artist.short.mbid, mbid);
        assert_eq!(artist.short.url, url);
        assert!(artist.bio.summary.is_some());
    }
}
