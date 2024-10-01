mod flac;
mod tag;

use isolang::Language;

use super::{Album, Artists, File, Genres, NameDateMbz, TrackDisc};
use crate::{config, Error};

pub trait Metadata<'a> {
    fn song(&'a self, config: &'a config::Parsing) -> Result<NameDateMbz<'a>, Error>;
    fn album(&'a self, config: &'a config::Parsing) -> Result<Album<'a>, Error>;
    fn artists(&'a self, config: &'a config::Parsing) -> Result<Artists<'a>, Error>;
    fn track_disc(&'a self, config: &'a config::Parsing) -> Result<TrackDisc, Error>;
    fn languages(&'a self, config: &'a config::Parsing) -> Result<Vec<Language>, Error>;
    fn genres(&'a self, config: &'a config::Parsing) -> Result<Genres<'a>, Error>;

    fn metadata(&'a self, config: &'a config::Parsing) -> Result<super::Metadata<'a>, Error> {
        Ok(super::Metadata {
            song: super::Song {
                main: self.song(config)?,
                track_disc: self.track_disc(config)?,
                languages: self.languages(config)?,
            },
            album: self.album(config)?,
            artists: self.artists(config)?,
            genres: self.genres(config)?,
        })
    }
}

pub trait Property {
    fn property(&self) -> Result<super::Property, Error>;
}

impl<'a> Metadata<'a> for File {
    fn song(&'a self, config: &'a config::Parsing) -> Result<NameDateMbz<'a>, Error> {
        match self {
            File::Flac { audio, .. } => audio.song(config),
        }
    }

    fn album(&'a self, config: &'a config::Parsing) -> Result<Album<'a>, Error> {
        match self {
            File::Flac { audio, .. } => audio.album(config),
        }
    }

    fn artists(&'a self, config: &'a config::Parsing) -> Result<Artists<'a>, Error> {
        match self {
            File::Flac { audio, .. } => audio.artists(config),
        }
    }

    fn track_disc(&'a self, config: &'a config::Parsing) -> Result<TrackDisc, Error> {
        match self {
            File::Flac { audio, .. } => audio.track_disc(config),
        }
    }

    fn languages(&'a self, config: &'a config::Parsing) -> Result<Vec<Language>, Error> {
        match self {
            File::Flac { audio, .. } => audio.languages(config),
        }
    }

    fn genres(&'a self, config: &'a config::Parsing) -> Result<Genres<'a>, Error> {
        match self {
            File::Flac { audio, .. } => audio.genres(config),
        }
    }
}

impl Property for File {
    fn property(&self) -> Result<super::Property, Error> {
        match self {
            File::Flac { audio, .. } => audio.property(),
        }
    }
}
