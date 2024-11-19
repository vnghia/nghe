use nghe_proc_macro::api_derive;

use super::Artist;
use crate::id3::album;

#[serde_with::apply(
    Vec => #[serde(skip_serializing_if = "Vec::is_empty")],
)]
#[api_derive(response = true)]
pub struct WithAlbums {
    #[serde(flatten)]
    pub artist: Artist,
    pub album: Vec<album::Album>,
}
