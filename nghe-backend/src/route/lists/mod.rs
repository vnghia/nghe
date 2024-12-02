mod get_album_list2;
mod get_random_songs;
mod get_songs_by_genre;
mod get_starred2;

nghe_proc_macro::build_router! {
    modules = [get_album_list2, get_random_songs, get_songs_by_genre, get_starred2],
}
