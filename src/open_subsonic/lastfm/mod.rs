mod scan_artist_lastfm_info;
mod utils;

use axum::Extension;
pub use scan_artist_lastfm_info::scan_artist_lastfm_info;

pub fn router(lastfm_client: Option<lastfm_client::Client>) -> axum::Router<crate::Database> {
    nghe_proc_macros::build_router!(scan_artist_lastfm_info).layer(Extension(lastfm_client))
}
