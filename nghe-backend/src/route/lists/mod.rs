mod get_album_list2;
mod get_random_songs;

nghe_proc_macro::build_router! {
    modules = [get_album_list2, get_random_songs],
}