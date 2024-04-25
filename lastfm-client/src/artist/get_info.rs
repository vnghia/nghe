use std::borrow::Cow;

use lastfm_proc_macros::MethodName;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use uuid::Uuid;

use super::Artist;

#[derive(Serialize, MethodName)]
pub struct Params<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mbid: Option<Uuid>,
}

#[serde_as]
#[derive(Deserialize)]
pub struct ArtistBio {
    #[serde_as(as = "serde_with::NoneAsEmptyString")]
    pub summary: Option<String>,
}

#[derive(Deserialize)]
pub struct ArtistFull {
    #[serde(flatten)]
    pub artist: Artist,
    pub bio: ArtistBio,
}

#[derive(Deserialize)]
pub struct Response {
    pub artist: ArtistFull,
}

#[cfg(all(test, lastfm_env))]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::Client;

    #[tokio::test]
    async fn test_artist_get_info() {
        let client = Client::new_from_env();
        let params = Params { artist: Some("Cher".into()), mbid: None };
        let artist = client.send::<_, Response>(&params).await.unwrap().artist;
        assert_eq!(artist.artist.name, "Cher");
        assert_eq!(
            artist.artist.mbid.unwrap(),
            Uuid::parse_str("bfcc6d75-a6a5-4bc6-8282-47aec8531818").unwrap()
        );
        assert_eq!(artist.artist.url, "https://www.last.fm/music/Cher");
        assert!(artist.bio.summary.is_some());
    }

    #[tokio::test]
    async fn test_artist_get_info_missing_mbid() {
        let client = Client::new_from_env();
        let params = Params { artist: Some("non-existent girl".into()), mbid: None };
        let artist = client.send::<_, Response>(&params).await.unwrap().artist;
        assert_eq!(artist.artist.name, "non-existent girl");
        assert!(artist.artist.mbid.is_none());
        assert_eq!(artist.artist.url, "https://www.last.fm/music/non-existent+girl");
    }
}
