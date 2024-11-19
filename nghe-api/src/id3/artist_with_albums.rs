use nghe_proc_macro::api_derive;

use super::{Album, Artist};

#[serde_with::apply(
    Option => #[serde(skip_serializing_if = "Option::is_none")],
    Vec => #[serde(skip_serializing_if = "Vec::is_empty")],
)]
#[api_derive(response = true)]
pub struct ArtistWithAlbum {
    #[serde(flatten)]
    pub artist: Artist,
    pub album: Vec<Album>,
}
