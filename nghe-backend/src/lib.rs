#![allow(incomplete_features)]
#![feature(adt_const_params)]
#![feature(anonymous_lifetime_in_impl_trait)]
#![feature(coverage_attribute)]
#![feature(duration_constructors)]
#![feature(if_let_guard)]
#![feature(iterator_try_collect)]
#![feature(let_chains)]
#![feature(proc_macro_hygiene)]
#![feature(specialization)]
#![feature(stmt_expr_attributes)]
#![feature(str_as_str)]
#![feature(try_blocks)]

#[coverage(off)]
pub mod config;
mod constant;
mod database;
#[coverage(off)]
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
#[coverage(off)]
mod test;

use axum::Router;
use error::Error;
use memory_serve::MemoryServe;
use mimalloc::MiMalloc;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::decompression::RequestDecompressionLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::services::Redirect;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[coverage(off)]
pub fn init_tracing(log: &config::Log) -> Result<(), Error> {
    color_eyre::install()?;

    let tracing = tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| if cfg!(test) { "debug" } else { "info" }.into()),
        )
        .with(tracing_error::ErrorLayer::default());

    let tracing_layer = tracing_subscriber::fmt::layer();

    if cfg!(test) {
        tracing.with(tracing_layer.with_test_writer()).try_init()?;
    } else if log.time {
        match log.format {
            config::log::Format::Plain => tracing.with(tracing_layer).try_init()?,
            config::log::Format::Json => tracing
                .with(tracing_layer.json().flatten_event(true).with_span_list(false))
                .try_init()?,
        }
    } else {
        let tracing_layer = tracing_layer.without_time();
        match log.format {
            config::log::Format::Plain => tracing.with(tracing_layer).try_init()?,
            config::log::Format::Json => tracing
                .with(tracing_layer.json().flatten_event(true).with_span_list(false))
                .try_init()?,
        }
    }

    Ok(())
}

#[coverage(off)]
pub async fn build(config: config::Config) -> Router {
    let filesystem =
        filesystem::Filesystem::new(&config.filesystem.tls, &config.filesystem.s3).await;
    let informant = integration::Informant::new(config.integration).await;

    let frontend_router = MemoryServe::from_env().fallback(Some("/index.html")).into_router();

    let backend_middleware = ServiceBuilder::new()
        .layer(RequestDecompressionLayer::new().br(true).gzip(true).zstd(true))
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(config.log.header))
                .on_request(())
                .on_response(DefaultOnResponse::new().include_headers(config.log.header)),
        )
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new().br(true).gzip(true).zstd(true));

    let backend_router = Router::new()
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
        .merge(route::key::router())
        .with_state(database::Database::new(&config.database))
        .layer(backend_middleware);

    Router::new()
        .nest_service(nghe_api::common::FRONTEND_PREFIX, frontend_router)
        .nest(nghe_api::common::BACKEND_PREFIX, backend_router)
        .fallback_service(Redirect::<axum::body::Body>::permanent(
            nghe_api::common::FRONTEND_PREFIX.parse().unwrap(),
        ))
}
