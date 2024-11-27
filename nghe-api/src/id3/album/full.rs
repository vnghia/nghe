use nghe_proc_macro::api_derive;

use super::Album;
use crate::id3::{artist, song};

#[api_derive]
pub struct Full {
    #[serde(flatten)]
    pub album: Album,
    pub artists: Vec<artist::Required>,
    pub is_compilation: bool,
    pub song: Vec<song::Song>,
}
