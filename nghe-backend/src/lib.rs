#![feature(adt_const_params)]
#![feature(anonymous_lifetime_in_impl_trait)]
#![feature(async_closure)]
#![feature(duration_constructors)]
#![feature(integer_sign_cast)]
#![feature(iterator_try_collect)]
#![feature(let_chains)]
#![feature(proc_macro_hygiene)]
#![feature(stmt_expr_attributes)]
#![feature(str_as_str)]
#![feature(try_blocks)]

mod auth;
pub mod config;
mod database;
mod error;
mod file;
mod filesystem;
mod http;
mod integration;
pub mod migration;
mod orm;
mod route;
mod scan;
mod schema;
mod sync;
mod time;
mod transcode;

#[cfg(test)]
mod test;

use axum::Router;
use error::Error;
use mimalloc::MiMalloc;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

pub async fn build(config: config::Config) -> Router {
    let filesystem =
        filesystem::Filesystem::new(&config.filesystem.tls, &config.filesystem.s3).await;
    let informant = integration::Informant::new(config.integration).await;

    Router::new()
        .merge(route::music_folder::router(filesystem.clone()))
        .merge(route::permission::router())
        .merge(route::user::router())
        .merge(route::media_retrieval::router(
            filesystem.clone(),
            config.transcode,
            config.cover_art.clone(),
        ))
        .merge(route::scan::router(
            filesystem,
            scan::scanner::Config {
                lofty: lofty::config::ParseOptions::default(),
                scan: config.filesystem.scan,
                parsing: config.parsing,
                index: config.index,
                cover_art: config.cover_art.clone(),
            },
            informant.clone(),
        ))
        .merge(route::bookmarks::router())
        .merge(route::browsing::router())
        .merge(route::lists::router())
        .merge(route::media_annotation::router(config.cover_art, informant))
        .merge(route::playlists::router())
        .merge(route::search::router())
        .merge(route::system::router())
        .with_state(database::Database::new(&config.database))
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new().br(true).gzip(true).zstd(true))
}
