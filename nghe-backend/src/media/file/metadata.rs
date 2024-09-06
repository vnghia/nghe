use isolang::Language;

use super::{artist, common, position};

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct Metadata<'a> {
    pub song: common::Common<'a>,
    pub album: common::Common<'a>,
    pub artists: artist::Artists<'a>,
    pub track_disc: position::TrackDisc,
    pub languages: Vec<Language>,
    pub genres: Vec<&'a str>,
    pub compilation: bool,
}
