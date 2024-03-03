mod create;
pub mod password;
mod setup;

use axum::{routing::get, Router};

pub fn router() -> Router<crate::Database> {
    Router::new()
        .route("/rest/setup", get(setup::setup_handler))
        .route("/rest/createUser", get(create::create_user_handler))
}

#[cfg(test)]
pub mod tests {
    pub use super::create::{create_user, CreateUserParams};
}
