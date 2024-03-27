mod album;
mod artist;
mod run_scan;
mod song;
mod start_scan;

use axum::routing::get;
use axum::{Extension, Router};
pub use start_scan::{start_scan, ScanMode, ScanStatistic};

use crate::config::parsing::ParsingConfig;
use crate::config::{ArtistIndexConfig, ScanConfig};

pub fn router(
    artist_index_config: ArtistIndexConfig,
    parsing_config: ParsingConfig,
    scan_config: ScanConfig,
) -> Router<crate::Database> {
    Router::new()
        .route("/rest/startScan", get(start_scan::start_scan_handler))
        .route("/rest/startScan.view", get(start_scan::start_scan_handler))
        .layer(Extension(artist_index_config))
        .layer(Extension(parsing_config))
        .layer(Extension(scan_config))
}

#[cfg(test)]
pub mod test {
    pub use super::album::upsert_album;
    pub use super::artist::upsert_artists;
}
