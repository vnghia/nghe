use crate::{config, Error};

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Client(rspotify::ClientCredsSpotify);

impl Client {
    pub async fn new(config: config::integration::Spotify) -> Result<Option<Self>, Error> {
        if let Some(id) = config.id {
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
            Ok(Some(Self(client)))
        } else {
            Ok(None)
        }
    }
}
