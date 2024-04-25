mod scan_artist_spotify_image;
mod update_artist_spotify_id;
mod utils;

use std::path::PathBuf;

use axum::Extension;
pub use scan_artist_spotify_image::scan_artist_spotify_image;

pub fn router(
    artist_art_path: Option<PathBuf>,
    spotify_client: Option<rspotify::ClientCredsSpotify>,
) -> axum::Router<crate::Database> {
    nghe_proc_macros::build_router!(scan_artist_spotify_image, update_artist_spotify_id)
        .layer(Extension(artist_art_path))
        .layer(Extension(spotify_client))
}
