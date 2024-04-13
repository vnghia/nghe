mod create;
mod delete;
mod get_basic_user_ids;
mod get_users;
mod login;
mod setup;

use axum::routing::get;
use axum::Router;

pub fn router() -> Router<crate::Database> {
    Router::new()
        .route("/rest/setup", get(setup::setup_handler))
        .route("/rest/setup.view", get(setup::setup_handler))
        .route("/rest/createUser", get(create::create_user_handler))
        .route("/rest/createUser.view", get(create::create_user_handler))
        .route("/rest/login", get(login::login_handler))
        .route("/rest/login.view", get(login::login_handler))
        .route("/rest/getUsers", get(get_users::get_users_handler))
        .route("/rest/getUsers.view", get(get_users::get_users_handler))
        .route("/rest/deleteUser", get(delete::delete_user_handler))
        .route("/rest/deleteUser.view", get(delete::delete_user_handler))
        .route("/rest/getBasicUserIds", get(get_basic_user_ids::get_basic_user_ids_handler))
        .route("/rest/getBasicUserIds.view", get(get_basic_user_ids::get_basic_user_ids_handler))
}

#[cfg(test)]
pub mod test {
    pub use super::create::create_user;
}
