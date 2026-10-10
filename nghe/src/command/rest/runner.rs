use axum_extra::headers::{self, HeaderMapExt};
use concat_string::concat_string;
use futures_lite::StreamExt;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use typed_path::{Utf8PlatformPath, Utf8PlatformPathBuf};
use url::Url;

use super::{Error, Rest, Route};
use crate::command::error;
use crate::config;

enum Output {
    File(tokio::fs::File),
    Stdout(tokio::io::Stdout),
}

pub struct Runner {
    url: Url,
    http: reqwest::Client,

    output: Option<Utf8PlatformPathBuf>,

    route: Route,
}

impl Output {
    async fn open(output: Option<&Utf8PlatformPath>) -> Result<Self, Error> {
        if let Some(output) = output {
            Ok(Self::File(tokio::fs::File::create(output).await?))
        } else {
            Ok(Self::Stdout(tokio::io::stdout()))
        }
    }
}

impl AsyncWrite for Output {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        match self.get_mut() {
            Self::File(file) => std::pin::pin!(file).poll_write(cx, buf),
            Self::Stdout(stdout) => std::pin::pin!(stdout).poll_write(cx, buf),
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            Self::File(file) => std::pin::pin!(file).poll_flush(cx),
            Self::Stdout(stdout) => std::pin::pin!(stdout).poll_flush(cx),
        }
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            Self::File(file) => std::pin::pin!(file).poll_shutdown(cx),
            Self::Stdout(stdout) => std::pin::pin!(stdout).poll_shutdown(cx),
        }
    }
}

impl TryFrom<Rest> for Runner {
    type Error = Error;

    fn try_from(rest: Rest) -> Result<Self, Self::Error> {
        let Rest { server, output, route } = rest;
        let config = config::Config::extract().ok();

        let url = if let Some(ref url) = server.url {
            url.to_owned()
        } else {
            let port = if let Some(config) = config {
                config.server.port
            } else {
                config::Server::default().port
            };
            concat_string!("http://localhost:", port.to_string()).parse()?
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

    async fn open_output(&self) -> Result<Output, Error> {
        Output::open(self.output.as_deref()).await
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
            error::Kind::Http { status_code: response.status(), error: response.json().await? }
                .into()
        }
    }

    pub async fn run_binary<'a, R: nghe_api::http::Request<'a, 'a, 'a, 'a, 'a>>(
        &self,
        request: Option<&R>,
    ) -> Result<(), Error> {
        let mut stream = self.send_url(request).await?.bytes_stream();
        let mut output = self.open_output().await?;
        while let Some(chunk) = stream.next().await {
            output.write_all(&chunk?).await?;
        }
        Ok(())
    }

    pub async fn run_endpoint<R: nghe_api::http::Endpoint>(
        &self,
        request: Option<&R>,
    ) -> Result<(), Error> {
        self.open_output()
            .await?
            .write_all(&serde_json::to_vec(
                &self.send_url(request).await?.json::<R::Response>().await?,
            )?)
            .await?;
        Ok(())
    }

    pub async fn run(&self) -> Result<(), Error> {
        self.route.run(self).await
    }
}
