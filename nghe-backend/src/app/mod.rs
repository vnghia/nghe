#![allow(clippy::needless_pass_by_value)]

use axum::body::Body;
use axum::http::Request;
use axum::Router;
use tower_http::trace::TraceLayer;

use crate::config::Config;

mod auth;
mod common;
pub mod migration;
pub mod music_folder;
pub mod permission;
pub mod state;
pub mod user;

pub async fn build(config: Config) -> Router {
    let filesystem = state::Filesystem::new(&config.filesystem.tls, &config.filesystem.s3).await;

    Router::new()
        .merge(music_folder::router(filesystem))
        .merge(permission::router())
        .merge(user::router())
        .with_state(state::App::new(&config.database))
        .layer(TraceLayer::new_for_http().make_span_with(|request: &Request<Body>| {
            tracing::info_span!(
                "request", method = %request.method(), uri = %request.uri()
            )
        }))
}
