mod ping;

use axum::{routing::get, Router};

use crate::ServerState;

pub fn router() -> Router<ServerState> {
    Router::new().route("/rest/ping", get(ping::ping_handler))
}
