mod model;

use concat_string::concat_string;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::{Error, error};

#[derive(Clone)]
pub struct Client {
    http: reqwest::Client,
    key: String,
}

trait Request: Serialize {
    type Response: DeserializeOwned;
    const NAME: &'static str;
}

impl Client {
    const LASTFM_ROOT_URL: &'static str = "https://ws.audioscrobbler.com/2.0/?";

    fn build_url<R: Request>(&self, request: &R) -> Result<String, Error> {
        serde_html_form::to_string(request)
            .map(|form| {
                concat_string!(
                    Self::LASTFM_ROOT_URL,
                    "method=",
                    R::NAME,
                    "&",
                    form,
                    "&",
                    "api_key=",
                    &self.key,
                    "&format=json"
                )
            })
            .map_err(|_| error::Kind::BuildLastFMRequestURLFailed.into())
    }

    async fn send<R: Request>(&self, request: &R) -> Result<R::Response, Error> {
        self.http
            .get(self.build_url(request)?)
            .send()
            .await?
            .error_for_status()?
            .json::<R::Response>()
            .await
            .map_err(Error::from)
    }
}
