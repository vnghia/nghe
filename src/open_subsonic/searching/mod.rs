pub mod search3;

pub fn router() -> axum::Router<crate::Database> {
    nghe_proc_macros::build_router!(search3)
}
