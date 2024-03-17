mod create;
mod setup;

use axum::{routing::get, Router};

pub fn router() -> Router<crate::Database> {
    Router::new()
        // setup
        .route("/rest/setup", get(setup::setup_handler))
        .route("/rest/setup.view", get(setup::setup_handler))
        // create_user
        .route("/rest/createUser", get(create::create_user_handler))
        .route("/rest/createUser.view", get(create::create_user_handler))
}

#[cfg(test)]
pub mod test {
    pub use super::create::{create_user, CreateUserParams};
}
