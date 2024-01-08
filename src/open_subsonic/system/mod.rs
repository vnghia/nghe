mod ping;

use axum::{routing::get, Router};

use crate::ServerState;

pub fn router(server_state: ServerState) -> Router<ServerState> {
    Router::new()
        .route("/rest/ping", get(ping::ping_handler))
        .with_state(server_state)
}
