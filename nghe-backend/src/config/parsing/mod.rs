pub mod vorbis_comments;

use serde::{Deserialize, Serialize};
use vorbis_comments::VorbisComments;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Parsing {
    pub vorbis_comments: VorbisComments,
}
