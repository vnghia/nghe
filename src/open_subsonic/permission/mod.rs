mod check_permission;
mod get_allowed_users;
mod set_permission;
mod with_permission;

use axum::routing::get;
use axum::Router;
pub use check_permission::check_permission;
pub use set_permission::set_permission;
pub use with_permission::with_permission;

pub fn router() -> Router<crate::Database> {
    Router::new()
        .route("/rest/getAllowedUsers", get(get_allowed_users::get_allowed_users_handler))
        .route("/rest/getAllowedUsers.view", get(get_allowed_users::get_allowed_users_handler))
        .route("/rest/setPermission", get(set_permission::set_permission_handler))
        .route("/rest/setPermission.view", get(set_permission::set_permission_handler))
}
