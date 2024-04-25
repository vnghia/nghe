use std::borrow::Cow;

use lastfm_proc_macros::MethodName;
use serde::{Deserialize, Serialize};

use super::Artist;

#[derive(Serialize, MethodName)]
pub struct Params<'a> {
    pub artist: Cow<'a, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u64>,
}

#[derive(Deserialize)]
pub struct ArtistMatches {
    pub artist: Vec<Artist>,
}

#[derive(Deserialize)]
pub struct Results {
    #[serde(rename = "artistmatches")]
    pub artist_matches: ArtistMatches,
}

#[derive(Deserialize)]
pub struct Response {
    pub results: Results,
}

#[cfg(all(test, lastfm_env))]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::Client;

    #[tokio::test]
    async fn test_artist_search() {
        let client = Client::new_from_env();
        let params = Params { artist: "cher".into(), limit: Some(1), page: Some(1) };
        let mut artists =
            client.send::<_, Response>(&params).await.unwrap().results.artist_matches.artist;
        assert_eq!(artists.len(), 1);
        let artist = artists.remove(0);
        assert_eq!(artist.name, "Cher");
        assert_eq!(
            artist.mbid.unwrap(),
            Uuid::parse_str("bfcc6d75-a6a5-4bc6-8282-47aec8531818").unwrap()
        );
        assert_eq!(artist.url, "https://www.last.fm/music/Cher");
    }
}
