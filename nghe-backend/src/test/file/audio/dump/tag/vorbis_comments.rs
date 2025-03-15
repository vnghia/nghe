use indexmap::IndexSet;
use isolang::Language;
use lofty::ogg::{OggPictureStorage, VorbisComments};
use uuid::Uuid;

use crate::config;
use crate::file::audio::position::Position;
use crate::file::audio::{Album, Artist, Artists, Date, Genres, NameDateMbz, TrackDisc};
use crate::file::lyric::Lyric;
use crate::file::image::Picture;
use crate::test::file::audio::dump;

impl Date {
    fn dump_vorbis_comments(self, tag: &mut VorbisComments, key: Option<&str>) {
        if let Some(key) = key
            && self.is_some()
        {
            tag.push(key.to_string(), self.to_string());
        }
    }
}

impl NameDateMbz<'_> {
    fn dump_vorbis_comments(
        self,
        tag: &mut VorbisComments,
        config: &config::parsing::vorbis_comments::Common,
    ) {
        let Self { name, date, release_date, original_release_date, mbz_id } = self;
        tag.push(config.name.clone(), name.into_owned());
        date.dump_vorbis_comments(tag, config.date.as_deref());
        release_date.dump_vorbis_comments(tag, config.release_date.as_deref());
        original_release_date.dump_vorbis_comments(tag, config.original_release_date.as_deref());
        if let Some(mbz_id) = mbz_id {
            tag.push(config.mbz_id.clone(), mbz_id.to_string());
        }
    }
}

impl Artist<'_> {
    fn dump_vorbis_comments(
        artists: IndexSet<Self>,
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

    fn dump_album(&mut self, config: &config::Parsing, album: Album<'_>) -> &mut Self {
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

    fn dump_genres(&mut self, config: &config::Parsing, genres: Genres<'_>) -> &mut Self {
        for genre in genres.value {
            self.push(config.vorbis_comments.genres.clone(), genre.value.into_owned());
        }
        self
    }

    fn dump_lyrics(&mut self, config: &config::Parsing, lyrics: Vec<Lyric<'_>>) -> &mut Self {
        for lyric in lyrics {
            self.push(
                if lyric.is_sync() {
                    &config.vorbis_comments.lyric.sync
                } else {
                    &config.vorbis_comments.lyric.unsync
                }
                .clone(),
                lyric.to_string(),
            );
        }
        self
    }

    fn dump_picture(&mut self, picture: Option<Picture<'_>>) -> &mut Self {
        if let Some(picture) = picture {
            self.insert_picture(picture.into(), None).unwrap();
        }
        self
    }
}
