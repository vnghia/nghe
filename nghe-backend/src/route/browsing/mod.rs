mod get_album;
mod get_album_info2;
mod get_artist;
mod get_artists;
mod get_genres;
mod get_music_folders;
mod get_song;
mod get_top_songs;

nghe_proc_macro::build_router! {
    modules = [
        get_album,
        get_album_info2,
        get_artist,
        get_artists,
        get_genres,
        get_music_folders,
        get_song,
        get_top_songs
    ],
}
