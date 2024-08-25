#![allow(clippy::needless_pass_by_value)]

use axum::body::Body;
use axum::http::Request;
use axum::Router;
use tower_http::trace::TraceLayer;

use crate::config::Config;

mod auth;
mod common;
mod error;
pub mod migration;
pub mod permission;
pub mod state;
pub mod user;

pub fn build(config: Config) -> Router {
    Router::new()
        .merge(permission::router())
        .merge(user::router())
        .with_state(state::App::new(&config.database))
        .layer(TraceLayer::new_for_http().make_span_with(|request: &Request<Body>| {
            tracing::info_span!(
                "request", method = %request.method(), uri = %request.uri()
            )
        }))
}
