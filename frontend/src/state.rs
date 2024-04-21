use std::borrow::Cow;

use anyhow::Result;
use concat_string::concat_string;
use dioxus::prelude::*;
use dioxus_sdk::storage::{use_synced_storage, LocalStorage};
use gloo::net::http::{Request, Response};
use nghe_types::error::ErrorSubsonicResponse;
use nghe_types::params::{CommonParams, WithCommon};
use nghe_types::response::{SubsonicResponse, SuccessConstantResponse};
use nghe_types::user::Role;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use crate::Route;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommonState {
    pub id: Uuid,
    pub common: CommonParams,
    pub role: Role,
    pub server_url: Option<Url>,
}

trait ResponseError {
    async fn error_for_status(self) -> Result<Self>
    where
        Self: Sized;
}

impl ResponseError for Response {
    async fn error_for_status(self) -> Result<Self> {
        if self.ok() {
            Ok(self)
        } else {
            let error = self.json::<ErrorSubsonicResponse>().await?.root.body.error;
            Err(anyhow::anyhow!("Opensubsonic error \"{}\" ({})", error.message, error.code))
        }
    }
}

impl CommonState {
    const COMMON_STATE_KEY: &'static str = "common-state";

    pub fn use_no_redirect() -> Signal<Option<Self>> {
        use_synced_storage::<LocalStorage, Option<Self>>(
            Self::COMMON_STATE_KEY.into(),
            Option::default,
        )
    }

    pub fn use_redirect() -> Signal<Option<Self>> {
        let nav = navigator();
        let common_state = Self::use_no_redirect();
        if common_state().is_none() {
            nav.push(Route::Login {});
        }
        common_state
    }

    pub fn build_url_with_common<'common, P: WithCommon<'common, Out = impl Serialize>>(
        &'common self,
        params: P,
    ) -> String {
        Self::build_url(params.with_common(&self.common))
    }

    pub fn build_url<P: Serialize>(params: P) -> String {
        serde_html_form::to_string(params)
            .expect("failed to serialize params which is not possible")
    }

    pub async fn send_with_common<
        'common,
        P: WithCommon<'common, Out = impl Serialize>,
        B: DeserializeOwned,
    >(
        &'common self,
        url: &'static str,
        params: P,
    ) -> Result<B> {
        Self::send_with_query(&self.server_url, url, &self.build_url_with_common(params)).await
    }

    pub async fn send<P: Serialize, B: DeserializeOwned>(
        server_url: &Option<Url>,
        url: &'static str,
        params: P,
    ) -> Result<B> {
        Self::send_with_query(server_url, url, &Self::build_url(params)).await
    }

    async fn send_with_query<B: DeserializeOwned>(
        server_url: &Option<Url>,
        url: &'static str,
        query: &str,
    ) -> Result<B> {
        Request::get(&concat_string!(
            server_url
                .as_ref()
                .map(|root_url| {
                    root_url
                        .join(url)
                        .expect("failed to join url which is not possible")
                        .to_string()
                        .into()
                })
                .unwrap_or(Cow::Borrowed(url)),
            "?",
            query
        ))
        .send()
        .await?
        .error_for_status()
        .await?
        .json::<SubsonicResponse<SuccessConstantResponse, B>>()
        .await
        .map(|r| r.root.body)
        .map_err(anyhow::Error::from)
    }
}
