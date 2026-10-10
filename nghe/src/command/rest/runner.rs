use axum_extra::headers::{self, HeaderMapExt};
use concat_string::concat_string;
use typed_path::Utf8PlatformPathBuf;
use url::Url;

use super::{Error, Rest, Route};
use crate::config;

pub struct Runner {
    url: Url,
    http: reqwest::Client,
    output: Option<Utf8PlatformPathBuf>,
    route: Route,
}

impl TryFrom<Rest> for Runner {
    type Error = Error;

    fn try_from(rest: Rest) -> Result<Self, Self::Error> {
        let Rest { server, output, route } = rest;
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

        Ok(Self {
            url,
            http: reqwest::ClientBuilder::new().default_headers(headers).build()?,
            output,
            route,
        })
    }
}

impl Runner {
    fn build_url<R: nghe_api::http::Url>(&self) -> Result<Url, Error> {
        self.url.join(&concat_string!(nghe_api::http::BACKEND_PREFIX, R::URL)).map_err(Error::from)
    }

    async fn send_url<R: nghe_api::http::Url>(
        &self,
        request: Option<&R>,
    ) -> Result<reqwest::Response, Error> {
        let url = self.build_url::<R>()?;
        let response = if let Some(request) = request {
            self.http.post(url).json(request).send().await
        } else {
            self.http.get(url).send().await
        }?;

        if response.status().is_success() {
            Ok(response)
        } else {
            Err(Error::Http { status_code: response.status(), error: response.json().await? })
        }
    }

    pub async fn run_binary<'a, R: nghe_api::http::Request<'a, 'a, 'a, 'a, 'a>>(
        &self,
        request: Option<&R>,
    ) -> Result<(), Error> {
        Ok(())
    }

    pub async fn run_endpoint<R: nghe_api::http::Endpoint>(
        &self,
        request: Option<&R>,
    ) -> Result<(), Error> {
        let response: R::Response = self.send_url(request).await?.json().await?;
        dbg!(response);
        Ok(())
    }

    pub async fn run(&self) -> Result<(), Error> {
        self.route.run(self).await
    }
}
