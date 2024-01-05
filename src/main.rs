mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

use axum::{response::Html, routing::get, Router};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use nghe::open_subsonic::system;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=info,tower_http=info", built_info::PKG_NAME).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app()).await.unwrap();
}

fn app() -> Router {
    Router::new()
        .route("/", get(handler))
        // system
        .route("/ping", get(system::ping))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}
