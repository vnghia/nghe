use anyhow::Error;
use gloo_net::http;
use nghe_api::common::{JsonEndpoint, JsonURL};

pub struct Client;

impl Client {
    pub async fn json<R: JsonEndpoint>(request: &R) -> Result<R::Response, Error> {
        let response = http::Request::post(<R as JsonURL>::URL_JSON).json(request)?.send().await?;
        if response.ok() {
            Ok(response.json().await?)
        } else {
            anyhow::bail!("{}", response.text().await?)
        }
    }
}
