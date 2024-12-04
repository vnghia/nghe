mod scrobble;
pub mod star;
pub mod unstar;
mod update_artist_information;

use crate::config;
use crate::integration::Informant;

nghe_proc_macro::build_router! {
    modules = [scrobble, star, unstar, update_artist_information(internal = true)],
    extensions = [config::CoverArt, Informant]
}
