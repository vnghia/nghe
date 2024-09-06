mod flac;
mod tag;

use std::borrow::Cow;

use enum_dispatch::enum_dispatch;
use isolang::Language;

use crate::config;
use crate::media::file::{Artists, Common, File, Metadata, TrackDisc};

#[enum_dispatch(File)]
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
