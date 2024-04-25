#![allow(clippy::too_many_arguments)]

mod album;
mod artist;
mod genre;
mod get_scan_status;
mod lyric;
mod run_scan;
mod song;
mod start_scan;

use axum::Extension;
pub use start_scan::{start_scan, ScanStat};

use crate::config::{ArtConfig, ArtistIndexConfig, ParsingConfig, ScanConfig};

pub fn router(
    artist_index_config: ArtistIndexConfig,
    parsing_config: ParsingConfig,
    scan_config: ScanConfig,
    art_config: ArtConfig,
    lastfm_client: Option<lastfm_client::Client>,
    spotify_client: Option<rspotify::ClientCredsSpotify>,
) -> axum::Router<crate::Database> {
    nghe_proc_macros::build_router!(start_scan, get_scan_status)
        .layer(Extension(artist_index_config))
        .layer(Extension(parsing_config))
        .layer(Extension(scan_config))
        .layer(Extension(art_config))
        .layer(Extension(lastfm_client))
        .layer(Extension(spotify_client))
}

#[cfg(test)]
pub mod test {
    pub use super::album::upsert_album;
    pub use super::artist::upsert_artists;
    pub use super::start_scan::initialize_scan;
}
