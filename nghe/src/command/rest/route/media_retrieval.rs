nghe_proc_macro::build_command! {
    module = media_retrieval,
    actions = [
        download(binary = true),
        get_cover_art(binary = true),
        get_lyrics_by_song_id(binary = true),
        stream(binary = true),
    ],
}
