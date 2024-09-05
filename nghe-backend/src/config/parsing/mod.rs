mod vorbis_comments;

use serde::{Deserialize, Serialize};
pub use vorbis_comments::{Artist, Artists, Common, TrackDisc, VorbisComments};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Parsing {
    pub vorbis_comments: VorbisComments,
}
