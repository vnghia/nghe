use std::borrow::Cow;

use lofty::flac::FlacFile;

use super::Metadata;
use crate::config;
use crate::media::audio::{Artists, NameDateMbz, TrackDisc};

impl Metadata for FlacFile {
    fn dump_song(&mut self, config: &config::Parsing, song: NameDateMbz<'_>) -> &mut Self {
        self.vorbis_comments_mut().unwrap().dump_song(config, song);
        self
    }

    fn dump_album(&mut self, config: &config::Parsing, album: NameDateMbz<'_>) -> &mut Self {
        self.vorbis_comments_mut().unwrap().dump_album(config, album);
        self
    }

    fn dump_artists(&mut self, config: &config::Parsing, artists: Artists<'_>) -> &mut Self {
        self.vorbis_comments_mut().unwrap().dump_artists(config, artists);
        self
    }

    fn dump_track_disc(&mut self, config: &config::Parsing, track_disc: TrackDisc) -> &mut Self {
        self.vorbis_comments_mut().unwrap().dump_track_disc(config, track_disc);
        self
    }

    fn dump_languages(
        &mut self,
        config: &config::Parsing,
        languages: Vec<isolang::Language>,
    ) -> &mut Self {
        self.vorbis_comments_mut().unwrap().dump_languages(config, languages);
        self
    }

    fn dump_genres(&mut self, config: &config::Parsing, genres: Vec<Cow<'_, str>>) -> &mut Self {
        self.vorbis_comments_mut().unwrap().dump_genres(config, genres);
        self
    }
}
