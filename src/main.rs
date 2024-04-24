use axum::Router;
use nghe::config::Config;
use nghe::open_subsonic::{
    bookmarks, browsing, extension, media_annotation, media_list, media_retrieval, music_folder,
    permission, playlists, scan, searching, system, user,
};
use nghe::Database;
use nghe_types::constant::{SERVER_NAME, SERVER_VERSION};
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            [constcat::concat!(SERVER_NAME, "=info").to_owned(), "tower_http=info".to_owned()]
                .join(",")
                .into()
        }))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!(server_version = SERVER_VERSION);

    let config = Config::default();
    tracing::info!(?config);

    // database
    let database = Database::new(&config.database.url, config.database.key).await;

    // run it
    let listener = tokio::net::TcpListener::bind(config.server.bind_addr).await.unwrap();
    tracing::info!(listening_addr = %listener.local_addr().unwrap());
    axum::serve(listener, app(database, config))
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

fn app(database: Database, config: Config) -> Router {
    let serve_frontend = ServeDir::new(&config.server.frontend_dir)
        .fallback(ServeFile::new(config.server.frontend_dir.join("index.html")));

    let lastfm_client = config.lastfm.key.map(lastfm_client::Client::new);

    Router::new()
        .merge(system::router())
        .merge(extension::router())
        .merge(browsing::router())
        .merge(user::router())
        .merge(media_retrieval::router(config.transcoding, config.art.clone()))
        .merge(media_list::router())
        .merge(bookmarks::router())
        .merge(searching::router())
        .merge(scan::router(
            config.artist_index,
            config.parsing,
            config.scan,
            config.art,
            lastfm_client,
        ))
        .merge(media_annotation::router())
        .merge(permission::router())
        .merge(music_folder::router())
        .merge(playlists::router())
        .layer(
            ServiceBuilder::new().layer(TraceLayer::new_for_http()).layer(CorsLayer::permissive()),
        )
        .fallback_service(serve_frontend)
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
