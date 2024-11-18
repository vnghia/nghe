mod get_artists;
mod get_music_folders;

nghe_proc_macro::build_router! {
    modules = [get_artists, get_music_folders],
}
