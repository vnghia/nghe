#![feature(coverage_attribute)]

use axum::serve::ListenerExt;
use nghe::{build, config, init_ffmpeg, init_tracing, migration};
use nghe_api::constant;

#[coverage(off)]
#[tokio::main]
async fn main() {
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
