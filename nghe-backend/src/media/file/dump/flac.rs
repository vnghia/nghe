use lofty::flac::FlacFile;

use super::MetadataDumper;
use crate::config;
use crate::media::file::{Artists, Common, TrackDisc};

impl MetadataDumper for FlacFile {
    fn dump_song(&mut self, config: &config::Parsing, song: Common<'_>) {
        self.vorbis_comments_mut().unwrap().dump_song(config, song);
    }

    fn dump_album(&mut self, config: &config::Parsing, album: Common<'_>) {
        self.vorbis_comments_mut().unwrap().dump_album(config, album);
    }

    fn dump_artists(&mut self, config: &config::Parsing, artists: Artists<'_>) {
        self.vorbis_comments_mut().unwrap().dump_artists(config, artists);
    }

    fn dump_track_disc(&mut self, config: &config::Parsing, track_disc: TrackDisc) {
        self.vorbis_comments_mut().unwrap().dump_track_disc(config, track_disc);
    }

    fn dump_languages(&mut self, config: &config::Parsing, languages: Vec<isolang::Language>) {
        self.vorbis_comments_mut().unwrap().dump_languages(config, languages);
    }

    fn dump_genres(&mut self, config: &config::Parsing, genres: Vec<&str>) {
        self.vorbis_comments_mut().unwrap().dump_genres(config, genres);
    }

    fn dump_compilation(&mut self, config: &config::Parsing, compilation: bool) {
        self.vorbis_comments_mut().unwrap().dump_compilation(config, compilation);
    }
}
