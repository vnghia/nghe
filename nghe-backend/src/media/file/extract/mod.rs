mod flac;
mod tag;

use std::borrow::Cow;

use isolang::Language;

use super::{Artists, File, NameDateMbz, TrackDisc};
use crate::{config, Error};

pub trait Metadata<'a> {
    fn song(&'a self, config: &'a config::Parsing) -> Result<NameDateMbz<'a>, Error>;
    fn album(&'a self, config: &'a config::Parsing) -> Result<NameDateMbz<'a>, Error>;
    fn artists(&'a self, config: &'a config::Parsing) -> Result<Artists<'a>, Error>;
    fn track_disc(&'a self, config: &'a config::Parsing) -> Result<TrackDisc, Error>;
    fn languages(&'a self, config: &'a config::Parsing) -> Result<Vec<Language>, Error>;
    fn genres(&'a self, config: &'a config::Parsing) -> Result<Vec<Cow<'a, str>>, Error>;
    fn compilation(&'a self, config: &'a config::Parsing) -> Result<bool, Error>;

    fn metadata(&'a self, config: &'a config::Parsing) -> Result<super::Metadata<'a>, Error> {
        Ok(super::Metadata {
            song: super::Song {
                main: self.song(config)?,
                track_disc: self.track_disc(config)?,
                languages: self.languages(config)?,
                genres: self.genres(config)?,
                compilation: self.compilation(config)?,
            },
            album: self.album(config)?,
            artists: self.artists(config)?,
        })
    }
}

pub trait Property {
    fn property(&self) -> Result<super::Property, Error>;
}

impl<'a> Metadata<'a> for File {
    fn song(&'a self, config: &'a config::Parsing) -> Result<NameDateMbz<'a>, Error> {
        match self {
            File::Flac { file, .. } => file.song(config),
        }
    }

    fn album(&'a self, config: &'a config::Parsing) -> Result<NameDateMbz<'a>, Error> {
        match self {
            File::Flac { file, .. } => file.album(config),
        }
    }

    fn artists(&'a self, config: &'a config::Parsing) -> Result<Artists<'a>, Error> {
        match self {
            File::Flac { file, .. } => file.artists(config),
        }
    }

    fn track_disc(&'a self, config: &'a config::Parsing) -> Result<TrackDisc, Error> {
        match self {
            File::Flac { file, .. } => file.track_disc(config),
        }
    }

    fn languages(&'a self, config: &'a config::Parsing) -> Result<Vec<Language>, Error> {
        match self {
            File::Flac { file, .. } => file.languages(config),
        }
    }

    fn genres(&'a self, config: &'a config::Parsing) -> Result<Vec<Cow<'a, str>>, Error> {
        match self {
            File::Flac { file, .. } => file.genres(config),
        }
    }

    fn compilation(&'a self, config: &'a config::Parsing) -> Result<bool, Error> {
        match self {
            File::Flac { file, .. } => file.compilation(config),
        }
    }
}

impl Property for File {
    fn property(&self) -> Result<super::Property, Error> {
        match self {
            File::Flac { file, .. } => file.property(),
        }
    }
}
