mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

use axum::Router;
use sea_orm_migration::prelude::*;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use nghe::config::Config;
use nghe::open_subsonic::{system, user};
use nghe::Migrator;
use nghe::ServerState;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                [
                    format!("{}=info", built_info::PKG_NAME),
                    "sea_orm_migration::migrator=info".to_owned(),
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
    let server_state = ServerState::new(config).await;

    // db migration
    Migrator::up(&server_state.conn, None)
        .await
        .expect("can not run pending migration(s)");

    // run it
    let listener = tokio::net::TcpListener::bind(format!(
        "{}:{}",
        server_state.config.server.host, server_state.config.server.port
    ))
    .await
    .unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app(server_state)).await.unwrap();
}

fn app(server_state: ServerState) -> Router {
    Router::new()
        // system
        .merge(system::router(server_state.clone()))
        // user
        .merge(user::router(server_state.clone()))
        // layer
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
        .with_state(server_state)
}
