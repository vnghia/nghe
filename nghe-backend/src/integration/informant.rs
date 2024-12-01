use super::spotify;
use crate::config;

#[derive(Debug, Clone)]
pub struct Informant {
    pub spotify: Option<spotify::Client>,
}

impl Informant {
    pub async fn new(config: config::Integration) -> Self {
        let spotify = spotify::Client::new(config.spotify).await;
        Self { spotify }
    }
}
