pub mod download;
mod get_cover_art;
mod stream;

use crate::config;

nghe_proc_macro::build_router! {
    modules = [download, get_cover_art, stream],
    filesystem = true,
    extensions = [config::Transcode, config::CoverArt],
}
