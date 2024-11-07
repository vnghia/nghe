pub mod download;
mod stream;

use crate::config;

nghe_proc_macro::build_router! {
    modules = [download, stream],
    filesystem = true,
    extensions = [config::Transcode],
}
