#![feature(coverage_attribute)]

use axum::serve::ListenerExt;
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

    let listener = tokio::net::TcpListener::bind(config.server.to_socket_addr())
        .await
        .unwrap()
        .tap_io(|tcp_stream| tcp_stream.set_nodelay(true).unwrap());
    axum::serve(listener, build(config).await).await.unwrap();
}
