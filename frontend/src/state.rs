use dioxus::prelude::*;
use dioxus_sdk::storage::{use_synced_storage, LocalStorage};
use nghe_types::open_subsonic::common::request::CommonParams;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::Route;

pub const COMMON_STATE_KEY: &str = "common-state";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommonState {
    pub common: CommonParams,
    pub server_url: Url,
}

pub fn use_common_state() -> Signal<Option<CommonState>> {
    let nav = navigator();
    let common_state = use_synced_storage::<LocalStorage, Option<CommonState>>(
        COMMON_STATE_KEY.into(),
        Option::default,
    );
    log::info!("{:?}", common_state());
    if common_state().is_none() {
        nav.push(Route::Login {});
    }
    common_state
}
