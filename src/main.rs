mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

use axum::Router;
use itertools::Itertools;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use nghe::config::Config;
use nghe::open_subsonic::{
    browsing,
    browsing::{refresh_music_folders, refresh_permissions},
    scan, system, user,
};
use nghe::Database;

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

    let config = Config::default();
    tracing::info!("configuration: {:?}", config);

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
    scan::run_scan(
        &database.pool,
        scan::ScanMode::Full,
        &upserted_music_folders,
    )
    .await
    .expect("can not scan song");

    // run it
    let listener = tokio::net::TcpListener::bind(config.server.bind_addr)
        .await
        .unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app(database)).await.unwrap();
}

fn app(database: Database) -> Router {
    Router::new()
        // system
        .merge(system::router())
        // browsing
        .merge(browsing::router())
        // user
        .merge(user::router())
        // layer
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
        .with_state(database)
}
