mod add_music_folder;
mod get_music_folder_ids;
mod get_music_folder_stat;
mod remove_music_folder;
mod update_music_folder;
mod utils;

use axum::routing::get;
use axum::Router;

pub fn router() -> Router<crate::Database> {
    Router::new()
        .route("/rest/getMusicFolderIds", get(get_music_folder_ids::get_music_folder_ids_handler))
        .route(
            "/rest/getMusicFolderIds.view",
            get(get_music_folder_ids::get_music_folder_ids_handler),
        )
        .route(
            "/rest/getMusicFolderStat",
            get(get_music_folder_stat::get_music_folder_stat_handler),
        )
        .route(
            "/rest/getMusicFolderStat.view",
            get(get_music_folder_stat::get_music_folder_stat_handler),
        )
        .route("/rest/addMusicFolder", get(add_music_folder::add_music_folder_handler))
        .route("/rest/addMusicFolder.view", get(add_music_folder::add_music_folder_handler))
        .route("/rest/updateMusicFolder", get(update_music_folder::update_music_folder_handler))
        .route(
            "/rest/updateMusicFolder.view",
            get(update_music_folder::update_music_folder_handler),
        )
        .route("/rest/removeMusicFolder", get(remove_music_folder::remove_music_folder_handler))
        .route(
            "/rest/removeMusicFolder.view",
            get(remove_music_folder::remove_music_folder_handler),
        )
}

#[cfg(test)]
pub mod test {
    pub use super::add_music_folder::add_music_folder;
}
