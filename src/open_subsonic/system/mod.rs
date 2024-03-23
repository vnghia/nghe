mod ping;

use axum::routing::get;
use axum::Router;

pub fn router() -> Router<crate::Database> {
    Router::new()
        .route("/rest/ping", get(ping::ping_handler))
        .route("/rest/ping.view", get(ping::ping_handler))
}
