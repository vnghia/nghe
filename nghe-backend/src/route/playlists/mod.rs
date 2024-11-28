pub mod create_playlist;
pub mod get_playlist;
pub mod get_playlists;

nghe_proc_macro::build_router! {
    modules = [create_playlist, get_playlist, get_playlists]
}
