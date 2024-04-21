mod add_permission;
mod check_permission;
mod get_allowed_users;
mod remove_permission;
mod with_permission;

pub use add_permission::add_permission;
pub use check_permission::check_permission;
pub use remove_permission::remove_permission;
pub use with_permission::with_permission;

pub fn router() -> axum::Router<crate::Database> {
    nghe_proc_macros::build_router!(get_allowed_users, add_permission, remove_permission)
}
