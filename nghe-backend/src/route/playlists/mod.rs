pub mod create_playlist;
mod delete_playlist;
pub mod get_playlist;
mod get_playlists;

nghe_proc_macro::build_router! {
    modules = [create_playlist, delete_playlist, get_playlist, get_playlists]
}
