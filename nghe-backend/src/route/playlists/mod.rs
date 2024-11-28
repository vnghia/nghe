pub mod create_playlist;
mod delete_playlist;
pub mod get_playlist;
mod get_playlists;
mod update_playlist;

nghe_proc_macro::build_router! {
    modules = [create_playlist, delete_playlist, get_playlist, get_playlists, update_playlist]
}
