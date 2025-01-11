use anyhow::Error;
use codee::string::{FromToStringCodec, OptionCodec};
use concat_string::concat_string;
use gloo_net::http;
use leptos::prelude::*;
use leptos_router::NavigateOptions;
use leptos_router::hooks::use_navigate;
use leptos_use::storage::use_local_storage;
use nghe_api::common::{JsonEndpoint, JsonURL};
use uuid::Uuid;

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

    pub fn use_client() -> (Signal<Option<Client>>, Effect<LocalStorage>) {
        let (read_api_key, _) = Self::use_api_key();
        let client = Signal::derive(move || read_api_key.with(|api_key| api_key.map(Client::new)));
        let effect = Effect::new(move |_| {
            if client.with(Option::is_none) {
                use_navigate()("/login", NavigateOptions::default());
            }
        });
        (client, effect)
    }

    async fn json_impl<R: JsonEndpoint>(
        request: &R,
        authorization: impl Into<Option<&str>>,
    ) -> Result<R::Response, Error> {
        let response = http::Request::post(<R as JsonURL>::URL_JSON)
            .header("Authorization", authorization.into().unwrap_or_default())
            .json(request)?
            .send()
            .await?;
        if response.ok() {
            Ok(response.json().await?)
        } else {
            anyhow::bail!("{}", response.text().await?)
        }
    }

    pub async fn json_no_auth<R: JsonEndpoint>(request: &R) -> Result<R::Response, Error> {
        Self::json_impl(request, None).await
    }
}
