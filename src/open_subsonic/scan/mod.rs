mod album;
mod artist;
mod genre;
mod get_scan_status;
mod lyric;
mod run_scan;
mod song;
mod start_scan;

use axum::routing::get;
use axum::{Extension, Router};
pub use start_scan::{start_scan, ScanStat};

use crate::config::{ArtConfig, ArtistIndexConfig, ParsingConfig, ScanConfig};

pub fn router(
    artist_index_config: ArtistIndexConfig,
    parsing_config: ParsingConfig,
    scan_config: ScanConfig,
    art_config: ArtConfig,
) -> Router<crate::Database> {
    Router::new()
        .route("/rest/startScan", get(start_scan::start_scan_handler))
        .route("/rest/startScan.view", get(start_scan::start_scan_handler))
        .route("/rest/getScanStatus", get(get_scan_status::get_scan_status_handler))
        .route("/rest/getScanStatus.view", get(get_scan_status::get_scan_status_handler))
        .layer(Extension(artist_index_config))
        .layer(Extension(parsing_config))
        .layer(Extension(scan_config))
        .layer(Extension(art_config))
}

#[cfg(test)]
pub mod test {
    pub use super::album::upsert_album;
    pub use super::artist::upsert_artists;
}
