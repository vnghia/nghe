mod create_user;
mod delete_user;
mod get_basic_user_ids;
mod get_users;
mod login;
mod setup;

pub fn router() -> axum::Router<crate::Database> {
    nghe_proc_macros::build_router!(
        setup,
        create_user,
        login,
        get_users,
        delete_user,
        get_basic_user_ids
    )
}

#[cfg(test)]
pub mod test {
    pub use super::create_user::create_user;
}
