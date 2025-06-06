use codee::string::{FromToStringCodec, OptionCodec};
use concat_string::concat_string;
use gloo_net::http;
use leptos::prelude::*;
use leptos_use::storage::use_local_storage;
use nghe_api::common::{JsonEndpoint, JsonURL};
use uuid::Uuid;

use crate::{Error, error};

#[derive(Clone)]
pub struct Client {
    authorization: String,
}

impl Client {
    const API_KEY_STORAGE_KEY: &'static str = "api-key";

    pub fn new(api_key: Uuid) -> Self {
        Self { authorization: concat_string!("Bearer ", api_key.to_string()) }
    }

    pub fn use_api_key() -> (Signal<Option<Uuid>>, WriteSignal<Option<Uuid>>) {
        let (read, write, _) = use_local_storage::<Option<Uuid>, OptionCodec<FromToStringCodec>>(
            Self::API_KEY_STORAGE_KEY,
        );
        (read, write)
    }

    pub fn use_client() -> Signal<Option<Client>> {
        let (read_api_key, _) = Self::use_api_key();
        Signal::derive(move || read_api_key.with(|api_key| api_key.map(Client::new)))
    }

    async fn json_impl<R: JsonEndpoint>(
        request: &R,
        authorization: Option<&str>,
    ) -> Result<R::Response, Error> {
        let response = http::Request::post(&concat_string!(
            nghe_api::common::BACKEND_PREFIX,
            <R as JsonURL>::URL_JSON
        ))
        .header("Authorization", authorization.unwrap_or_default())
        .json(request)?
        .send()
        .await?;
        if response.ok() {
            Ok(response.json().await?)
        } else {
            let code = response.status();
            let text = response.text().await?;
            Err(if text.is_empty() {
                error::Http { code, text: response.status_text() }.into()
            } else {
                error::Http { code, text }.into()
            })
        }
    }

    pub async fn json_no_auth<R: JsonEndpoint>(request: &R) -> Result<R::Response, Error> {
        Self::json_impl(request, None).await
    }

    pub async fn json<R: JsonEndpoint>(&self, request: &R) -> Result<R::Response, Error> {
        Self::json_impl(request, Some(&self.authorization)).await
    }
}
