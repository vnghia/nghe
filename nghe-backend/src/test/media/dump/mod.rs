mod flac;
mod tag;

use std::borrow::Cow;

use isolang::Language;

use crate::config;
use crate::media::file::{self, Artists, File, NameDateMbz, TrackDisc};

pub trait Metadata {
    fn dump_song(&mut self, config: &config::Parsing, song: NameDateMbz<'_>) -> &mut Self;
    fn dump_album(&mut self, config: &config::Parsing, album: NameDateMbz<'_>) -> &mut Self;
    fn dump_artists(&mut self, config: &config::Parsing, artists: Artists<'_>) -> &mut Self;
    fn dump_track_disc(&mut self, config: &config::Parsing, track_disc: TrackDisc) -> &mut Self;
    fn dump_languages(&mut self, config: &config::Parsing, languages: Vec<Language>) -> &mut Self;
    fn dump_genres(&mut self, config: &config::Parsing, genres: Vec<Cow<'_, str>>) -> &mut Self;
    fn dump_compilation(&mut self, config: &config::Parsing, compilation: bool) -> &mut Self;

    fn dump_metadata(
        &mut self,
        config: &config::Parsing,
        metadata: file::Metadata<'_>,
    ) -> &mut Self {
        let file::Metadata { song, album, artists, genres } = metadata;
        let file::Song { main, track_disc, languages, compilation } = song;
        self.dump_song(config, main)
            .dump_album(config, album)
            .dump_artists(config, artists)
            .dump_track_disc(config, track_disc)
            .dump_languages(config, languages)
            .dump_genres(config, genres)
            .dump_compilation(config, compilation)
    }
}

impl Metadata for File {
    fn dump_song(&mut self, config: &config::Parsing, song: NameDateMbz<'_>) -> &mut Self {
        match self {
            File::Flac { file, .. } => {
                file.dump_song(config, song);
            }
        }
        self
    }

    fn dump_album(&mut self, config: &config::Parsing, album: NameDateMbz<'_>) -> &mut Self {
        match self {
            File::Flac { file, .. } => {
                file.dump_album(config, album);
            }
        }
        self
    }

    fn dump_artists(&mut self, config: &config::Parsing, artists: Artists<'_>) -> &mut Self {
        match self {
            File::Flac { file, .. } => {
                file.dump_artists(config, artists);
            }
        }
        self
    }

    fn dump_track_disc(&mut self, config: &config::Parsing, track_disc: TrackDisc) -> &mut Self {
        match self {
            File::Flac { file, .. } => {
                file.dump_track_disc(config, track_disc);
            }
        }
        self
    }

    fn dump_languages(&mut self, config: &config::Parsing, languages: Vec<Language>) -> &mut Self {
        match self {
            File::Flac { file, .. } => {
                file.dump_languages(config, languages);
            }
        }
        self
    }

    fn dump_genres(&mut self, config: &config::Parsing, genres: Vec<Cow<'_, str>>) -> &mut Self {
        match self {
            File::Flac { file, .. } => {
                file.dump_genres(config, genres);
            }
        }
        self
    }

    fn dump_compilation(&mut self, config: &config::Parsing, compilation: bool) -> &mut Self {
        match self {
            File::Flac { file, .. } => {
                file.dump_compilation(config, compilation);
            }
        }
        self
    }
}
