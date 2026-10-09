use axum_extra::headers::{self, HeaderMapExt};
use concat_string::concat_string;
use url::Url;

use crate::command::Error;
use crate::config;

pub struct Client {
    url: Url,
    inner: reqwest::Client,
}

impl Client {
    pub fn new(server: &super::Server) -> Result<Self, Error> {
        let config = config::Config::try_extract().ok();

        let url = if let Some(ref url) = server.url {
            url.to_owned()
        } else {
            let port = if let Some(config) = config {
                config.server.port
            } else {
                config::Server::default().port
            };
            concat_string!("http://localhost:", port.to_string())
                .parse()
                .expect("This is a valid url")
        };

        let mut headers = reqwest::header::HeaderMap::new();
        if let Some(ref username) = server.auth.username
            && let Some(ref password) = server.auth.password
        {
            headers.typed_insert(headers::Authorization::basic(username, password));
        } else if let Some(ref api_key) = server.auth.api_key {
            headers.typed_insert(headers::Authorization::bearer(api_key)?);
        }

        Ok(Self { url, inner: reqwest::ClientBuilder::new().default_headers(headers).build()? })
    }

    fn build_url<R: nghe_api::http::Url>(&self) -> Result<Url, Error> {
        self.url
            .join(&concat_string!(nghe_api::http::BACKEND_PREFIX, "/", R::URL))
            .map_err(Error::from)
    }

    async fn send_url<R: nghe_api::http::Url>(
        &self,
        request: Option<&R>,
    ) -> Result<reqwest::Response, Error> {
        let url = self.build_url::<R>()?;
        let response = if let Some(request) = request {
            self.inner.post(url).json(request).send().await
        } else {
            self.inner.get(url).send().await
        }?;

        if response.status().is_success() {
            Ok(response)
        } else {
            Err(Error::Http {
                status_code: response.status(),
                error: serde_json::from_slice(&response.bytes().await?)?,
            })
        }
    }

    pub async fn send_endpoint<R: nghe_api::http::Endpoint>(
        &self,
        request: Option<&R>,
    ) -> Result<R::Response, Error> {
        Ok(serde_json::from_slice(&self.send_url(request).await?.bytes().await?)?)
    }
}
