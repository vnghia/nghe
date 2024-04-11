use dioxus::prelude::*;
use dioxus_sdk::storage::{use_synced_storage, LocalStorage};
use nghe_types::params::CommonParams;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::Route;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommonState {
    pub common: CommonParams,
    pub server_url: Url,
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
}
