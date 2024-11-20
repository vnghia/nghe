mod get_album;
mod get_artist;
mod get_artists;
mod get_music_folders;

nghe_proc_macro::build_router! {
    modules = [get_album, get_artist, get_artists, get_music_folders],
}
