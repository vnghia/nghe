mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

use axum::Router;
use itertools::Itertools;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use nghe::config::Config;
use nghe::migration;
use nghe::open_subsonic::{
    browsing,
    browsing::{refresh_music_folders, refresh_permissions},
    scan::scan_full,
    system, user,
};
use nghe::ServerState;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                [
                    format!("{}=info", built_info::PKG_NAME),
                    "tower_http=info".to_owned(),
                ]
                .join(",")
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::new().expect("configuration can not be parsed");
    tracing::info!("configuration: {:?}", config);

    // state
    let server_state = ServerState::new(&config).await;

    // db migration
    migration::run_pending_migrations(&config.database.url).await;

    // music folders
    let (upserted_music_folders, _) = refresh_music_folders(
        &server_state.database.pool,
        &config.folder.top_paths,
        &config.folder.depth_levels,
    )
    .await;

    // user music folders
    refresh_permissions(
        &server_state.database.pool,
        None,
        Some(
            &upserted_music_folders
                .iter()
                .map(|music_folder| music_folder.id)
                .collect_vec(),
        ),
    )
    .await
    .expect("can not set music folders permissions");

    // scan song
    scan_full(
        &server_state.database.pool,
        &server_state.artist.ignored_prefixes,
        &upserted_music_folders,
    )
    .await
    .expect("can not scan song");

    // run it
    let listener =
        tokio::net::TcpListener::bind(format!("{}:{}", config.server.host, config.server.port))
            .await
            .unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app(server_state)).await.unwrap();
}

fn app(server_state: ServerState) -> Router {
    Router::new()
        // system
        .merge(system::router(server_state.clone()))
        // browsing
        .merge(browsing::router(server_state.clone()))
        // user
        .merge(user::router(server_state.clone()))
        // layer
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
        .with_state(server_state)
}
