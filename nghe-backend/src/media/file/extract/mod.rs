mod flac;
mod tag;

use std::borrow::Cow;

use enum_dispatch::enum_dispatch;
use isolang::Language;

use super::{Artists, Common, Metadata, Property, TrackDisc};
use crate::{config, Error};

#[enum_dispatch(File)]
pub trait MetadataExtractor {
    fn song<'a>(&'a self, config: &'a config::Parsing) -> Result<Common<'a>, Error>;
    fn album<'a>(&'a self, config: &'a config::Parsing) -> Result<Common<'a>, Error>;
    fn artists<'a>(&'a self, config: &'a config::Parsing) -> Result<Artists<'a>, Error>;
    fn track_disc<'a>(&'a self, config: &'a config::Parsing) -> Result<TrackDisc, Error>;
    fn languages<'a>(&'a self, config: &'a config::Parsing) -> Result<Vec<Language>, Error>;
    fn genres<'a>(&'a self, config: &'a config::Parsing) -> Result<Vec<Cow<'a, str>>, Error>;
    fn compilation<'a>(&'a self, config: &'a config::Parsing) -> Result<bool, Error>;

    fn metadata<'a>(&'a self, config: &'a config::Parsing) -> Result<Metadata<'a>, Error> {
        Ok(Metadata {
            song: self.song(config)?,
            album: self.album(config)?,
            artists: self.artists(config)?,
            track_disc: self.track_disc(config)?,
            languages: self.languages(config)?,
            genres: self.genres(config)?,
            compilation: self.compilation(config)?,
        })
    }
}

#[enum_dispatch(File)]
pub trait PropertyExtractor {
    fn property(&self) -> Result<Property, Error>;
}
