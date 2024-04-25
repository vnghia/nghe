pub mod scan_artist_lastfm_info;
pub mod utils;

use axum::Extension;

pub fn router(lastfm_client: Option<lastfm_client::Client>) -> axum::Router<crate::Database> {
    nghe_proc_macros::build_router!(scan_artist_lastfm_info).layer(Extension(lastfm_client))
}
