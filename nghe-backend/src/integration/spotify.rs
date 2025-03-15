use rspotify::clients::BaseClient;
use rspotify::model::{ArtistId, SearchResult, SearchType};

use crate::{Error, config, error};

#[derive(Debug, Clone)]
pub struct Artist {
    pub id: ArtistId<'static>,
    pub image_url: Option<String>,
}

#[derive(Clone)]
#[repr(transparent)]
pub struct Client(rspotify::ClientCredsSpotify);

impl Client {
    pub async fn new(config: config::integration::Spotify) -> Option<Self> {
        if let Some(id) = config.id {
            tracing::info!("spotify integration enabled");
            let creds = rspotify::Credentials { id, secret: config.secret };
            let config = if let Some(token_path) = config.token_path {
                tokio::fs::create_dir_all(
                    token_path.parent().expect("Could not get parent directory for spotify token"),
                )
                .await
                .expect("Could not create directory for spotify token");
                rspotify::Config {
                    token_cached: true,
                    cache_path: token_path.into(),
                    ..Default::default()
                }
            } else {
                rspotify::Config { token_cached: false, ..Default::default() }
            };
            let client = rspotify::ClientCredsSpotify::with_config(creds, config);
            client.request_token().await.expect("Could not authenticate to spotify server");
            Some(Self(client))
        } else {
            None
        }
    }

    #[cfg_attr(
        not(coverage_nightly),
        tracing::instrument(skip_all, name = "spotify:search_artist", ret(level = "debug"))
    )]
    pub async fn search_artist(&self, name: &str) -> Result<Option<Artist>, Error> {
        Ok(
            if let SearchResult::Artists(artists) =
                self.0.search(name, SearchType::Artist, None, None, Some(1), Some(0)).await?
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

    pub async fn fetch_artist(&self, id: &str) -> Result<Artist, Error> {
        self.0
            .artist(
                ArtistId::from_id(id)
                    .map_err(|_| error::Kind::InvalidSpotifyIdFormat(id.to_owned()))?,
            )
            .await
            .map(|artist| Artist {
                id: artist.id,
                image_url: artist.images.into_iter().next().map(|image| image.url),
            })
            .map_err(Error::from)
    }
}

#[cfg(all(test, spotify_env))]
#[coverage(off)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::file::image;

    #[rstest]
    #[case("Micheal Learns To Rock")]
    #[tokio::test]
    async fn test_search_artist(#[case] name: &str) {
        let client = Client::new(config::integration::Spotify::from_env()).await.unwrap();
        let artist = client.search_artist(name).await.unwrap().unwrap();
        picture::Picture::fetch(&reqwest::Client::default(), artist.image_url.unwrap())
            .await
            .unwrap();
    }
}
