pub mod download;
mod get_cover_art;
mod get_lyrics_by_song_id;
mod stream;

use crate::config;

nghe_proc_macro::build_router! {
    modules = [download, get_cover_art, get_lyrics_by_song_id, stream],
    filesystem = true,
    extensions = [config::Transcode, config::CoverArt],
}
