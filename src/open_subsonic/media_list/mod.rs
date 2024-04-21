mod get_album_list2;
mod get_starred2;

pub fn router() -> axum::Router<crate::Database> {
    nghe_proc_macros::build_router!(get_starred2, get_album_list2)
}
