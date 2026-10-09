use axum::Router;
use axum::serve::ListenerExt as _;
use nghe_api::constant;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::decompression::RequestDecompressionLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::services::Redirect;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::{Error, config, database, filesystem, integration, migration, route, scan};

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
pub fn init_ffmpeg(transcode: &config::Transcode) -> Result<(), Error> {
    ffmpeg_next::init()?;
    ffmpeg_next::util::log::set_level(transcode.log_level.into());
    Ok(())
}

#[coverage(off)]
pub async fn build(config: config::Config) -> Router {
    let filesystem = filesystem::Filesystem::new(&config.filesystem.tls, &config.filesystem.s3);
    let informant = integration::Informant::new(config.integration).await;

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
        .merge(route::playlist::router())
        .merge(route::search::router())
        .merge(route::system::router())
        .merge(route::key::router())
        .with_state(database::Database::new(&config.database))
        .layer(backend_middleware);

    Router::new().nest(nghe_api::http::BACKEND_PREFIX, backend_router).fallback_service(Redirect::<
        axum::body::Body,
    >::permanent(
        nghe_api::http::FRONTEND_PREFIX.parse().unwrap(),
    ))
}

#[coverage(off)]
pub async fn start() {
    let config = config::Config::default();
    init_tracing(&config.log).unwrap();
    init_ffmpeg(&config.transcode).unwrap();

    tracing::info!(server_version =% constant::SERVER_VERSION);
    tracing::info!("{config:?}");

    migration::run(&config.database.url).await;

    let listener = tokio::net::TcpListener::bind(config.server.to_socket_addr())
        .await
        .unwrap()
        .tap_io(|tcp_stream| tcp_stream.set_nodelay(true).unwrap());
    axum::serve(listener, build(config).await).await.unwrap();
}
