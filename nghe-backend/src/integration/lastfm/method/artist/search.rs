use serde::{Deserialize, Serialize};

use crate::Error;
use crate::integration::lastfm;
use crate::integration::lastfm::model::artist;

#[serde_with::apply(
    Option => #[serde(skip_serializing_if = "Option::is_none")]
)]
#[derive(Debug, Serialize)]
struct Request<'a> {
    artist: &'a str,
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
    pub async fn search_artist(
        &self,
        artist: impl AsRef<str>,
    ) -> Result<Option<artist::Short>, Error> {
        self.send(&Request { artist: artist.as_ref(), limit: Some(1), page: None })
            .await
            .map(|response| response.results.artist_matches.artist.into_iter().next())
    }
}

#[cfg(all(test, lastfm_env))]
#[coverage(off)]
mod tests {
    use rstest::rstest;
    use uuid::{Uuid, uuid};

    use super::*;
    use crate::config;

    #[rstest]
    #[case(
        "Cher",
        Some(uuid!("bfcc6d75-a6a5-4bc6-8282-47aec8531818")),
        "https://www.last.fm/music/Cher",
    )]
    #[tokio::test]
    async fn test_search_artist(#[case] name: &str, #[case] mbid: Option<Uuid>, #[case] url: &str) {
        let client = lastfm::Client::new(
            reqwest::Client::default(),
            config::integration::Lastfm::from_env(),
        )
        .unwrap();
        let artist = client.search_artist(name).await.unwrap().unwrap();
        assert_eq!(artist.name, name);
        assert_eq!(artist.mbid, mbid);
        assert_eq!(artist.url, url);
    }
}
