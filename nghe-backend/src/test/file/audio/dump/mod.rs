mod flac;
mod tag;

use std::borrow::Cow;

use isolang::Language;

use crate::config;
use crate::file::audio::{self, Artists, File, NameDateMbz, TrackDisc};

pub trait Metadata {
    fn dump_song(&mut self, config: &config::Parsing, song: NameDateMbz<'_>) -> &mut Self;
    fn dump_album(&mut self, config: &config::Parsing, album: NameDateMbz<'_>) -> &mut Self;
    fn dump_artists(&mut self, config: &config::Parsing, artists: Artists<'_>) -> &mut Self;
    fn dump_track_disc(&mut self, config: &config::Parsing, track_disc: TrackDisc) -> &mut Self;
    fn dump_languages(&mut self, config: &config::Parsing, languages: Vec<Language>) -> &mut Self;
    fn dump_genres(&mut self, config: &config::Parsing, genres: Vec<Cow<'_, str>>) -> &mut Self;

    fn dump_metadata(
        &mut self,
        config: &config::Parsing,
        metadata: audio::Metadata<'_>,
    ) -> &mut Self {
        let audio::Metadata { song, album, artists, genres } = metadata;
        let audio::Song { main, track_disc, languages } = song;
        self.dump_song(config, main)
            .dump_album(config, album)
            .dump_artists(config, artists)
            .dump_track_disc(config, track_disc)
            .dump_languages(config, languages)
            .dump_genres(config, genres)
    }
}

impl Metadata for File {
    fn dump_song(&mut self, config: &config::Parsing, song: NameDateMbz<'_>) -> &mut Self {
        match self {
            File::Flac { audio, .. } => {
                audio.dump_song(config, song);
            }
        }
        self
    }

    fn dump_album(&mut self, config: &config::Parsing, album: NameDateMbz<'_>) -> &mut Self {
        match self {
            File::Flac { audio, .. } => {
                audio.dump_album(config, album);
            }
        }
        self
    }

    fn dump_artists(&mut self, config: &config::Parsing, artists: Artists<'_>) -> &mut Self {
        match self {
            File::Flac { audio, .. } => {
                audio.dump_artists(config, artists);
            }
        }
        self
    }

    fn dump_track_disc(&mut self, config: &config::Parsing, track_disc: TrackDisc) -> &mut Self {
        match self {
            File::Flac { audio, .. } => {
                audio.dump_track_disc(config, track_disc);
            }
        }
        self
    }

    fn dump_languages(&mut self, config: &config::Parsing, languages: Vec<Language>) -> &mut Self {
        match self {
            File::Flac { audio, .. } => {
                audio.dump_languages(config, languages);
            }
        }
        self
    }

    fn dump_genres(&mut self, config: &config::Parsing, genres: Vec<Cow<'_, str>>) -> &mut Self {
        match self {
            File::Flac { audio, .. } => {
                audio.dump_genres(config, genres);
            }
        }
        self
    }
}
