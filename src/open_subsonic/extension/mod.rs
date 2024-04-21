mod get_open_subsonic_extensions;

pub fn router() -> axum::Router<crate::Database> {
    nghe_proc_macros::build_router!(get_open_subsonic_extensions)
}
