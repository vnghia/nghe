use axum::extract::{FromRef, FromRequest, Request};
use nghe_api::common::Auth;
use serde::de::DeserializeOwned;

use super::error::Error;
use super::state::App;

#[derive(Debug)]
pub struct Get<R> {
    pub request: R,
}

#[async_trait::async_trait]
impl<R, S> FromRequest<S> for Get<R>
where
    R: DeserializeOwned + Send,
    S: Send + Sync,
    App: FromRef<S>,
{
    type Rejection = Error;

    #[tracing::instrument(skip_all, err)]
    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let query =
            request.uri().query().ok_or_else(|| Error::BadRequest("missing query parameters"))?;

        // TODO: Optimize this after https://github.com/serde-rs/serde/issues/1183
        let auth: Auth = serde_html_form::from_str(query)
            .map_err(|_| Error::BadRequest("invalid auth parameters"))?;
        let request: R = serde_html_form::from_str(query)
            .map_err(|_| Error::BadRequest("invalid request parameters"))?;

        Ok(Self { request })
    }
}
