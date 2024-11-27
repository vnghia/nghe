use nghe_proc_macro::api_derive;

use super::Artist;
use crate::id3::album;

#[api_derive]
pub struct Full {
    #[serde(flatten)]
    pub artist: Artist,
    pub album: Vec<album::Album>,
}
