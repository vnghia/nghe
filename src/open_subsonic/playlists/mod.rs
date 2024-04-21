mod add_playlist_user;
mod create_playlist;
mod delete_playlist;
mod get_playlist;
mod get_playlists;
mod id3;
mod update_playlist;
mod utils;

pub fn router() -> axum::Router<crate::Database> {
    nghe_proc_macros::build_router!(
        create_playlist,
        get_playlists,
        get_playlist,
        add_playlist_user,
        update_playlist,
        delete_playlist
    )
}
