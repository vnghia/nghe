pub mod create;
pub mod password;

use axum::{routing::get, Router};

use crate::ServerState;

pub fn router(server_state: ServerState) -> Router<ServerState> {
    Router::new()
        .route("/rest/createUser", get(create::create_user_handler))
        .with_state(server_state)
}
