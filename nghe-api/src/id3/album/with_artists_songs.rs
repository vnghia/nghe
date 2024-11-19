use nghe_proc_macro::api_derive;

use super::Album;
use crate::id3::{artist, song};

#[serde_with::apply(
    Vec => #[serde(skip_serializing_if = "Vec::is_empty")],
)]
#[api_derive(response = true)]
pub struct WithArtistsSongs {
    #[serde(flatten)]
    pub album: Album,
    pub artists: Vec<artist::Artist>,
    pub is_compilation: bool,
    pub song: Vec<song::Song>,
}
