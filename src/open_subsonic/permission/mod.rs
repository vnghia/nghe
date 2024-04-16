mod add_permission;
mod check_permission;
mod get_allowed_users;
mod remove_permission;
mod with_permission;

pub use add_permission::add_permission;
use axum::routing::get;
use axum::Router;
pub use check_permission::check_permission;
pub use remove_permission::remove_permission;
pub use with_permission::with_permission;

pub fn router() -> Router<crate::Database> {
    Router::new()
        .route("/rest/getAllowedUsers", get(get_allowed_users::get_allowed_users_handler))
        .route("/rest/getAllowedUsers.view", get(get_allowed_users::get_allowed_users_handler))
        .route("/rest/addPermission", get(add_permission::add_permission_handler))
        .route("/rest/addPermission.view", get(add_permission::add_permission_handler))
        .route("/rest/removePermission", get(remove_permission::remove_permission_handler))
        .route("/rest/removePermission.view", get(remove_permission::remove_permission_handler))
}
