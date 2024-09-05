mod vorbis_comments;

use isolang::Language;

use super::{Artists, Common, TrackDisc};
use crate::{config, Error};

pub trait Metadata<'a> {
    fn song(&'a self, config: &'a config::Parsing) -> Result<Common<'a>, Error>;
    fn album(&'a self, config: &'a config::Parsing) -> Result<Common<'a>, Error>;
    fn artists(&'a self, config: &'a config::Parsing) -> Result<Artists<'a>, Error>;
    fn track_disc(&'a self, config: &'a config::Parsing) -> Result<TrackDisc, Error>;
    fn languages(&'a self, config: &'a config::Parsing) -> Result<Vec<Language>, Error>;
    fn genres(&'a self, config: &'a config::Parsing) -> Vec<&'a str>;
    fn compilation(&'a self, config: &'a config::Parsing) -> bool;

    fn extract(&'a self, config: &'a config::Parsing) -> Result<super::Metadata<'a>, Error> {
        Ok(super::Metadata {
            song: self.song(config)?,
            album: self.album(config)?,
            artists: self.artists(config)?,
            track_disc: self.track_disc(config)?,
            languages: self.languages(config)?,
            genres: self.genres(config),
            compilation: self.compilation(config),
        })
    }
}
