use std::borrow::Cow;

use isolang::Language;
use lofty::ogg::VorbisComments;
use uuid::Uuid;

use crate::config;
use crate::media::audio::position::Position;
use crate::media::audio::{Artist, Artists, Date, NameDateMbz, TrackDisc};
use crate::test::media::dump;

impl Date {
    fn dump_vorbis_comments(self, tag: &mut VorbisComments, key: &Option<String>) {
        if let Some(key) = key
            && self.is_some()
        {
            tag.push(key.clone(), self.to_string());
        }
    }
}

impl<'a> NameDateMbz<'a> {
    fn dump_vorbis_comments(
        self,
        tag: &mut VorbisComments,
        config: &config::parsing::vorbis_comments::Common,
    ) {
        let Self { name, date, release_date, original_release_date, mbz_id } = self;
        tag.push(config.name.clone(), name.into_owned());
        date.dump_vorbis_comments(tag, &config.date);
        release_date.dump_vorbis_comments(tag, &config.release_date);
        original_release_date.dump_vorbis_comments(tag, &config.original_release_date);
        if let Some(mbz_id) = mbz_id {
            tag.push(config.mbz_id.clone(), mbz_id.to_string());
        }
    }
}

impl<'a> Artist<'a> {
    fn dump_vorbis_comments(
        artists: Vec<Self>,
        tag: &mut VorbisComments,
        config: &config::parsing::vorbis_comments::Artist,
    ) {
        for artist in artists {
            tag.push(config.name.clone(), artist.name.into_owned());
            tag.push(config.mbz_id.clone(), artist.mbz_id.unwrap_or(Uuid::nil()).to_string());
        }
    }
}

impl Position {
    fn dump_vorbis_comments(self, tag: &mut VorbisComments, number_key: &str, total_key: &str) {
        if let Some(number) = self.number {
            tag.push(number_key.to_owned(), number.to_string());
        }
        if let Some(total) = self.total {
            tag.push(total_key.to_owned(), total.to_string());
        }
    }
}

impl dump::Metadata for VorbisComments {
    fn dump_song(&mut self, config: &config::Parsing, song: NameDateMbz<'_>) -> &mut Self {
        song.dump_vorbis_comments(self, &config.vorbis_comments.song);
        self
    }

    fn dump_album(&mut self, config: &config::Parsing, album: NameDateMbz<'_>) -> &mut Self {
        album.dump_vorbis_comments(self, &config.vorbis_comments.album);
        self
    }

    fn dump_artists(&mut self, config: &config::Parsing, artists: Artists<'_>) -> &mut Self {
        Artist::dump_vorbis_comments(artists.song, self, &config.vorbis_comments.artists.song);
        Artist::dump_vorbis_comments(artists.album, self, &config.vorbis_comments.artists.album);
        if artists.compilation {
            self.push(config.vorbis_comments.compilation.clone(), "1".to_string());
        }
        self
    }

    fn dump_track_disc(&mut self, config: &config::Parsing, track_disc: TrackDisc) -> &mut Self {
        track_disc.track.dump_vorbis_comments(
            self,
            &config.vorbis_comments.track_disc.track_number,
            &config.vorbis_comments.track_disc.track_total,
        );
        track_disc.disc.dump_vorbis_comments(
            self,
            &config.vorbis_comments.track_disc.disc_number,
            &config.vorbis_comments.track_disc.disc_total,
        );
        self
    }

    fn dump_languages(&mut self, config: &config::Parsing, languages: Vec<Language>) -> &mut Self {
        for language in languages {
            self.push(config.vorbis_comments.languages.clone(), language.to_string());
        }
        self
    }

    fn dump_genres(&mut self, config: &config::Parsing, genres: Vec<Cow<'_, str>>) -> &mut Self {
        for genre in genres {
            self.push(config.vorbis_comments.genres.clone(), genre.into_owned());
        }
        self
    }
}
