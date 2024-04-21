mod get_album;
mod get_album_info2;
mod get_artist;
mod get_artist_info2;
mod get_artists;
mod get_genres;
mod get_indexes;
mod get_music_directory;
mod get_music_folders;
mod get_song;
mod get_top_songs;

pub fn router() -> axum::Router<crate::Database> {
    nghe_proc_macros::build_router!(
        get_music_folders,
        get_artists,
        get_artist,
        get_album,
        get_song,
        get_indexes,
        get_music_directory,
        get_genres,
        get_top_songs,
        get_album_info2,
        get_artist_info2
    )
}
