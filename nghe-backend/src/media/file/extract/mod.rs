mod flac;
mod tag;

use std::borrow::Cow;

use isolang::Language;

use super::{Artists, Common, File, Metadata, Property, TrackDisc};
use crate::{config, Error};

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

pub trait PropertyExtractor {
    fn property(&self) -> Result<Property, Error>;
}

impl MetadataExtractor for File {
    fn song<'a>(&'a self, config: &'a config::Parsing) -> Result<Common<'a>, Error> {
        match self {
            File::Flac(file) => file.song(config),
        }
    }

    fn album<'a>(&'a self, config: &'a config::Parsing) -> Result<Common<'a>, Error> {
        match self {
            File::Flac(file) => file.album(config),
        }
    }

    fn artists<'a>(&'a self, config: &'a config::Parsing) -> Result<Artists<'a>, Error> {
        match self {
            File::Flac(file) => file.artists(config),
        }
    }

    fn track_disc<'a>(&'a self, config: &'a config::Parsing) -> Result<TrackDisc, Error> {
        match self {
            File::Flac(file) => file.track_disc(config),
        }
    }

    fn languages<'a>(&'a self, config: &'a config::Parsing) -> Result<Vec<Language>, Error> {
        match self {
            File::Flac(file) => file.languages(config),
        }
    }

    fn genres<'a>(&'a self, config: &'a config::Parsing) -> Result<Vec<Cow<'a, str>>, Error> {
        match self {
            File::Flac(file) => file.genres(config),
        }
    }

    fn compilation<'a>(&'a self, config: &'a config::Parsing) -> Result<bool, Error> {
        match self {
            File::Flac(file) => file.compilation(config),
        }
    }
}

impl PropertyExtractor for File {
    fn property(&self) -> Result<Property, Error> {
        match self {
            File::Flac(file) => file.property(),
        }
    }
}
