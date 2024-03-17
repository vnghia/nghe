mod ping;

use axum::{routing::get, Router};

pub fn router() -> Router<crate::Database> {
    Router::new()
        // view
        .route("/rest/ping", get(ping::ping_handler))
        .route("/rest/ping.view", get(ping::ping_handler))
}
