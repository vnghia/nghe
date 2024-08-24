#![feature(const_mut_refs)]

use nghe_api::constant;
use nghe_backend::build;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() {
    color_eyre::install().expect("Could not install error handler");

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            ["nghe_backend=info".to_owned(), "tower_http=info".to_owned()].join(",").into()
        }))
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .try_init()
        .expect("Could not install tracing handler");

    tracing::info!(server_version = constant::SERVER_VERSION);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, build()).await.unwrap();
}
