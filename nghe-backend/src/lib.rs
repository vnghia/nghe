#![feature(adt_const_params)]
#![feature(anonymous_lifetime_in_impl_trait)]
#![feature(const_mut_refs)]
#![feature(iterator_try_collect)]
#![feature(let_chains)]
#![feature(try_blocks)]

mod auth;
pub mod config;
mod database;
mod error;
mod filesystem;
mod media;
pub mod migration;
mod orm;
mod route;
mod scan;
mod schema;

#[cfg(test)]
mod test;

use axum::body::Body;
use axum::http::Request;
use axum::Router;
use error::Error;
use tower_http::trace::TraceLayer;

pub async fn build(config: config::Config) -> Router {
    let filesystem =
        filesystem::Filesystem::new(&config.filesystem.tls, &config.filesystem.s3).await;

    Router::new()
        .merge(route::music_folder::router(filesystem))
        .merge(route::permission::router())
        .merge(route::user::router())
        .with_state(database::Database::new(&config.database))
        .layer(TraceLayer::new_for_http().make_span_with(|request: &Request<Body>| {
            tracing::info_span!(
                "request", method = %request.method(), uri = %request.uri()
            )
        }))
}
