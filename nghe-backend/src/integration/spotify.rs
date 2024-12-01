use rspotify::clients::BaseClient;
use rspotify::model::{ArtistId, SearchResult, SearchType};

use crate::{config, Error};

#[derive(Debug, Clone)]
pub struct Artist {
    pub id: ArtistId<'static>,
    pub image_url: Option<String>,
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Client(rspotify::ClientCredsSpotify);

impl Client {
    pub async fn new(config: config::integration::Spotify) -> Result<Option<Self>, Error> {
        Ok(if let Some(id) = config.id {
            tracing::info!("Spotify integration enabled");
            let creds = rspotify::Credentials { id, secret: config.secret };
            let config = if let Some(token_path) = config.token_path {
                rspotify::Config {
                    token_cached: true,
                    cache_path: token_path.into(),
                    ..Default::default()
                }
            } else {
                rspotify::Config { token_cached: false, ..Default::default() }
            };
            let client = rspotify::ClientCredsSpotify::with_config(creds, config);
            client.request_token().await?;
            Some(Self(client))
        } else {
            None
        })
    }

    pub async fn search_artist(&self, name: &str) -> Result<Option<Artist>, Error> {
        Ok(
            if let SearchResult::Artists(artists) =
                self.0.search(name, SearchType::Artist, None, None, None, None).await?
            {
                artists.items.into_iter().next().map(|artist| Artist {
                    id: artist.id,
                    image_url: artist.images.into_iter().next().map(|image| image.url),
                })
            } else {
                None
            },
        )
    }
}

#[cfg(all(test, spotify_env))]
mod tests {
    use concat_string::concat_string;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("Micheal Learn To Rock", "3fMbdgg4jU18AjLCKBhRSm")]
    #[tokio::test]
    async fn test_search_artist(#[case] name: &str, #[case] id: &str) {
        let client = Client::new(config::integration::Spotify::from_env()).await.unwrap().unwrap();
        let artist = client.search_artist(name).await.unwrap().unwrap();
        assert_eq!(artist.id.to_string(), concat_string!("spotify:artist:", id));
        assert!(artist.image_url.is_some());
    }
}
