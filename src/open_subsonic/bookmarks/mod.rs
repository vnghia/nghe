mod get_bookmarks;

pub fn router() -> axum::Router<crate::Database> {
    nghe_proc_macros::build_router!(get_bookmarks)
}
