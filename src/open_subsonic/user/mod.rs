pub mod create;
pub mod password;
pub mod setup;

use axum::{routing::get, Router};

pub fn router() -> Router<crate::Database> {
    Router::new()
        .route("/rest/setup", get(setup::setup_handler))
        .route("/rest/createUser", get(create::create_user_handler))
}
