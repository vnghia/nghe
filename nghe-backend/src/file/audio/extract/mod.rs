mod flac;
mod mpeg;
mod tag;

use isolang::Language;

use super::{Album, Artists, File, Genres, NameDateMbz, TrackDisc};
use crate::file::picture::Picture;
use crate::{Error, config};

pub trait Metadata<'a> {
    fn song(&'a self, config: &'a config::Parsing) -> Result<NameDateMbz<'a>, Error>;
    fn album(&'a self, config: &'a config::Parsing) -> Result<Album<'a>, Error>;
    fn artists(&'a self, config: &'a config::Parsing) -> Result<Artists<'a>, Error>;
    fn track_disc(&'a self, config: &'a config::Parsing) -> Result<TrackDisc, Error>;
    fn languages(&'a self, config: &'a config::Parsing) -> Result<Vec<Language>, Error>;
    fn genres(&'a self, config: &'a config::Parsing) -> Result<Genres<'a>, Error>;
    fn picture(&'a self) -> Result<Option<Picture<'static, 'a>>, Error>;

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
            picture: self.picture()?,
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
            File::Mpeg { audio, .. } => audio.song(config),
        }
    }

    fn album(&'a self, config: &'a config::Parsing) -> Result<Album<'a>, Error> {
        match self {
            File::Flac { audio, .. } => audio.album(config),
            File::Mpeg { audio, .. } => audio.album(config),
        }
    }

    fn artists(&'a self, config: &'a config::Parsing) -> Result<Artists<'a>, Error> {
        match self {
            File::Flac { audio, .. } => audio.artists(config),
            File::Mpeg { audio, .. } => audio.artists(config),
        }
    }

    fn track_disc(&'a self, config: &'a config::Parsing) -> Result<TrackDisc, Error> {
        match self {
            File::Flac { audio, .. } => audio.track_disc(config),
            File::Mpeg { audio, .. } => audio.track_disc(config),
        }
    }

    fn languages(&'a self, config: &'a config::Parsing) -> Result<Vec<Language>, Error> {
        match self {
            File::Flac { audio, .. } => audio.languages(config),
            File::Mpeg { audio, .. } => audio.languages(config),
        }
    }

    fn genres(&'a self, config: &'a config::Parsing) -> Result<Genres<'a>, Error> {
        match self {
            File::Flac { audio, .. } => audio.genres(config),
            File::Mpeg { audio, .. } => audio.genres(config),
        }
    }

    fn picture(&'a self) -> Result<Option<Picture<'static, 'a>>, Error> {
        match self {
            File::Flac { audio, .. } => audio.picture(),
            File::Mpeg { audio, .. } => audio.picture(),
        }
    }
}

impl Property for File {
    fn property(&self) -> Result<super::Property, Error> {
        match self {
            File::Flac { audio, .. } => audio.property(),
            File::Mpeg { audio, .. } => audio.property(),
        }
    }
}
