mod ping;

use axum::{routing::get, Router};

pub fn router() -> Router<crate::Database> {
    Router::new().route("/rest/ping", get(ping::ping_handler))
}
