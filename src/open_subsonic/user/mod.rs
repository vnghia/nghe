pub mod create;
pub mod password;
pub mod setup;

use axum::{routing::get, Router};

use crate::ServerState;

pub fn router(server_state: ServerState) -> Router<ServerState> {
    Router::new()
        .route("/rest/setup", get(setup::setup_handler))
        .route("/rest/createUser", get(create::create_user_handler))
        .with_state(server_state)
}
