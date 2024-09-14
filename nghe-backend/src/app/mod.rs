#![allow(clippy::needless_pass_by_value)]

use axum::body::Body;
use axum::http::Request;
use axum::Router;
use tower_http::trace::TraceLayer;

use crate::config::Config;
use crate::filesystem::Filesystem;

mod auth;
mod common;
pub mod route;
pub mod state;

pub async fn build(config: Config) -> Router {
    let filesystem = Filesystem::new(&config.filesystem.tls, &config.filesystem.s3).await;

    Router::new()
        .merge(route::music_folder::router(filesystem))
        .merge(route::permission::router())
        .merge(route::user::router())
        .with_state(state::App::new(&config.database))
        .layer(TraceLayer::new_for_http().make_span_with(|request: &Request<Body>| {
            tracing::info_span!(
                "request", method = %request.method(), uri = %request.uri()
            )
        }))
}

#[cfg(test)]
pub mod test {
    pub mod permission {
        pub use super::super::route::permission::test::*;
    }
}
