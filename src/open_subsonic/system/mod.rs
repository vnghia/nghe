mod ping;

pub fn router() -> axum::Router<crate::Database> {
    nghe_proc_macros::build_router!(ping)
}
