#![feature(coverage_attribute)]

use nghe_api::constant;
use nghe_backend::{build, config, init_tracing, migration};

#[coverage(off)]
#[tokio::main]
async fn main() {
    init_tracing().unwrap();
    tracing::info!(server_version =% constant::SERVER_VERSION);

    let config = config::Config::default();
    tracing::info!("{config:#?}");
    migration::run(&config.database.url).await;

    let listener = tokio::net::TcpListener::bind(config.server.to_socket_addr()).await.unwrap();
    axum::serve(listener, build(config).await).tcp_nodelay(true).await.unwrap();
}
