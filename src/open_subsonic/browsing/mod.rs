pub mod get_artist;
pub mod get_music_folders;
pub mod refresh_music_folders;
pub mod refresh_permissions;

pub use refresh_music_folders::refresh_music_folders;
pub use refresh_permissions::refresh_permissions;

use axum::{routing::get, Router};

pub fn router() -> Router<crate::Database> {
    Router::new().route(
        "/rest/getMusicFolders",
        get(get_music_folders::get_music_folders_handler),
    )
}
