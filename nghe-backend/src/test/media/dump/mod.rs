mod flac;
mod tag;

use std::borrow::Cow;

use isolang::Language;

use crate::config;
use crate::media::file::{Artists, Common, File, Metadata, TrackDisc};

pub trait MetadataDumper {
    fn dump_song(&mut self, config: &config::Parsing, song: Common<'_>);
    fn dump_album(&mut self, config: &config::Parsing, album: Common<'_>);
    fn dump_artists(&mut self, config: &config::Parsing, artists: Artists<'_>);
    fn dump_track_disc(&mut self, config: &config::Parsing, track_disc: TrackDisc);
    fn dump_languages(&mut self, config: &config::Parsing, languages: Vec<Language>);
    fn dump_genres(&mut self, config: &config::Parsing, genres: Vec<Cow<'_, str>>);
    fn dump_compilation(&mut self, config: &config::Parsing, compilation: bool);

    fn dump_metadata(&mut self, config: &config::Parsing, metadata: Metadata<'_>) {
        let Metadata { song, album, artists, track_disc, languages, genres, compilation } =
            metadata;
        self.dump_song(config, song);
        self.dump_album(config, album);
        self.dump_artists(config, artists);
        self.dump_track_disc(config, track_disc);
        self.dump_languages(config, languages);
        self.dump_genres(config, genres);
        self.dump_compilation(config, compilation);
    }
}

impl MetadataDumper for File {
    fn dump_song(&mut self, config: &config::Parsing, song: Common<'_>) {
        match self {
            File::Flac(file) => file.dump_song(config, song),
        }
    }

    fn dump_album(&mut self, config: &config::Parsing, album: Common<'_>) {
        match self {
            File::Flac(file) => file.dump_album(config, album),
        }
    }

    fn dump_artists(&mut self, config: &config::Parsing, artists: Artists<'_>) {
        match self {
            File::Flac(file) => file.dump_artists(config, artists),
        }
    }

    fn dump_track_disc(&mut self, config: &config::Parsing, track_disc: TrackDisc) {
        match self {
            File::Flac(file) => file.dump_track_disc(config, track_disc),
        }
    }

    fn dump_languages(&mut self, config: &config::Parsing, languages: Vec<Language>) {
        match self {
            File::Flac(file) => file.dump_languages(config, languages),
        }
    }

    fn dump_genres(&mut self, config: &config::Parsing, genres: Vec<Cow<'_, str>>) {
        match self {
            File::Flac(file) => file.dump_genres(config, genres),
        }
    }

    fn dump_compilation(&mut self, config: &config::Parsing, compilation: bool) {
        match self {
            File::Flac(file) => file.dump_compilation(config, compilation),
        }
    }
}
