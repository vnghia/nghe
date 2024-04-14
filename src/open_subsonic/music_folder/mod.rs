mod get_folder_stats;

use axum::routing::get;
use axum::Router;

pub fn router() -> Router<crate::Database> {
    Router::new()
        .route("/rest/getFolderStats", get(get_folder_stats::get_folder_stats_handler))
        .route("/rest/getFolderStats.view", get(get_folder_stats::get_folder_stats_handler))
}
