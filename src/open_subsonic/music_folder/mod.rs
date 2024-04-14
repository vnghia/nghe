mod get_music_folder_stats;

use axum::routing::get;
use axum::Router;

pub fn router() -> Router<crate::Database> {
    Router::new()
        .route(
            "/rest/getMusicFolderStats",
            get(get_music_folder_stats::get_music_folder_stats_handler),
        )
        .route(
            "/rest/getMusicFolderStats.view",
            get(get_music_folder_stats::get_music_folder_stats_handler),
        )
}
