use isolang::Language;

use super::{artist, common, position};

#[derive(Debug)]
pub struct Metadata<'a> {
    pub song: common::Common<'a>,
    pub album: common::Common<'a>,
    pub artist: artist::SongAlbum<'a>,
    pub track_disc: position::TrackDisc,
    pub languages: Vec<Language>,
    pub genres: Vec<String>,
    pub compilation: bool,
}
