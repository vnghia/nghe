mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

use axum::Router;
use itertools::Itertools;
use nghe::config::Config;
use nghe::open_subsonic::browsing::{refresh_music_folders, refresh_permissions};
use nghe::open_subsonic::{
    bookmarks, browsing, extension, media_list, media_retrieval, scan, searching, system, user,
};
use nghe::Database;
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            [
                constcat::concat!(built_info::PKG_NAME, "=info").to_owned(),
                "tower_http=info".to_owned(),
            ]
            .join(",")
            .into()
        }))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::default();
    tracing::info!(?config);

    // database
    let database = Database::new(&config.database.url, config.database.key).await;

    // music folders
    let (upserted_music_folders, _) = refresh_music_folders(
        &database.pool,
        &config.folder.top_paths,
        &config.folder.depth_levels,
    )
    .await;

    // user music folders
    refresh_permissions(
        &database.pool,
        None,
        Some(&upserted_music_folders.iter().map(|music_folder| music_folder.id).collect_vec()),
    )
    .await
    .expect("can not set music folders permissions");

    // scan song
    scan::start_scan(
        &database.pool,
        scan::ScanMode::Full,
        &upserted_music_folders,
        &config.artist_index,
        &config.parsing,
        &config.scan,
        &config.art,
    )
    .await
    .expect("can not scan song");

    // run it
    let listener = tokio::net::TcpListener::bind(config.server.bind_addr).await.unwrap();
    tracing::info!(listening_addr = %listener.local_addr().unwrap());
    axum::serve(listener, app(database, config))
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

fn app(database: Database, config: Config) -> Router {
    Router::new()
        .merge(system::router())
        .merge(extension::router())
        .merge(browsing::router())
        .merge(user::router())
        .merge(media_retrieval::router(config.transcoding, config.art.clone()))
        .merge(media_list::router())
        .merge(bookmarks::router())
        .merge(searching::router())
        .merge(scan::router(config.artist_index, config.parsing, config.scan, config.art))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
        .with_state(database)
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
