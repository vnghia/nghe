use concat_string::concat_string;
use indexmap::IndexSet;
use isolang::Language;
use itertools::Itertools;
use lofty::TextEncoding;
use lofty::id3::v2::{Id3v2Tag, TextInformationFrame, TimestampFrame};
use uuid::Uuid;

use crate::config;
use crate::config::parsing::id3v2::frame;
use crate::file::audio::position::Position;
use crate::file::audio::{Album, Artist, Artists, Date, Genres, NameDateMbz, TrackDisc};
use crate::file::picture::Picture;
use crate::test::file::audio::dump;

fn write_text(tag: &mut Id3v2Tag, frame_id: frame::Id, text: String) {
    match frame_id {
        frame::Id::Text(frame_id) => {
            tag.insert(TextInformationFrame::new(frame_id, TextEncoding::UTF8, text).into())
        }
        frame::Id::UserText(description) => tag.insert_user_text(description, text),
        frame::Id::Time(_) => unreachable!(),
    };
}

fn write_texts(
    tag: &mut Id3v2Tag,
    frame_id: frame::Id,
    texts: impl Iterator<Item = impl ToString>,
) {
    write_text(
        tag,
        frame_id,
        texts.map(|text| text.to_string()).join(&frame::Id::ID3V24_SEPARATOR.to_string()),
    );
}

impl Date {
    fn dump_id3v2(self, tag: &mut Id3v2Tag, frame_id: Option<frame::Id>) {
        if let Some(frame_id) = frame_id
            && self.is_some()
        {
            match frame_id {
                frame::Id::Time(frame_id) => {
                    if let Some(timestamp) = self.into() {
                        tag.insert(
                            TimestampFrame::new(frame_id, TextEncoding::UTF8, timestamp).into(),
                        );
                    }
                }
                _ => write_text(tag, frame_id, self.to_string()),
            }
        }
    }
}

impl NameDateMbz<'_> {
    fn dump_id3v2(self, tag: &mut Id3v2Tag, config: config::parsing::id3v2::Common) {
        let Self { name, date, release_date, original_release_date, mbz_id } = self;
        write_text(tag, config.name, name.into_owned());
        date.dump_id3v2(tag, config.date);
        release_date.dump_id3v2(tag, config.release_date);
        original_release_date.dump_id3v2(tag, config.original_release_date);
        if let Some(mbz_id) = mbz_id {
            write_text(tag, config.mbz_id, mbz_id.to_string());
        }
    }
}

impl Artist<'_> {
    fn dump_id3v2(
        artists: IndexSet<Self>,
        tag: &mut Id3v2Tag,
        config: config::parsing::id3v2::Artist,
    ) {
        let (names, mbz_ids): (Vec<_>, Vec<_>) = artists
            .into_iter()
            .map(|artist| {
                (artist.name.into_owned(), artist.mbz_id.unwrap_or(Uuid::nil()).to_string())
            })
            .collect();
        write_texts(tag, config.name, names.into_iter());
        write_texts(tag, config.mbz_id, mbz_ids.into_iter());
    }
}

impl Position {
    fn dump_id3v2(self, tag: &mut Id3v2Tag, frame_id: frame::Id) {
        if self.number.is_some() || self.total.is_some() {
            write_text(
                tag,
                frame_id,
                concat_string!(
                    self.number.as_ref().map(u16::to_string).unwrap_or_default(),
                    "/",
                    self.total.as_ref().map(u16::to_string).unwrap_or_default()
                ),
            );
        }
    }
}

impl dump::Metadata for Id3v2Tag {
    fn dump_song(&mut self, config: &config::Parsing, song: NameDateMbz<'_>) -> &mut Self {
        song.dump_id3v2(self, config.id3v2.song.clone());
        self
    }

    fn dump_album(&mut self, config: &config::Parsing, album: Album<'_>) -> &mut Self {
        album.dump_id3v2(self, config.id3v2.album.clone());
        self
    }

    fn dump_artists(&mut self, config: &config::Parsing, artists: Artists<'_>) -> &mut Self {
        Artist::dump_id3v2(artists.song, self, config.id3v2.artists.song.clone());
        Artist::dump_id3v2(artists.album, self, config.id3v2.artists.album.clone());
        if artists.compilation {
            write_text(self, config.id3v2.compilation.clone(), "1".to_string());
        }
        self
    }

    fn dump_track_disc(&mut self, config: &config::Parsing, track_disc: TrackDisc) -> &mut Self {
        track_disc.track.dump_id3v2(self, config.id3v2.track_disc.track_position.clone());
        track_disc.disc.dump_id3v2(self, config.id3v2.track_disc.disc_position.clone());
        self
    }

    fn dump_languages(&mut self, config: &config::Parsing, languages: Vec<Language>) -> &mut Self {
        write_texts(self, config.id3v2.languages.clone(), languages.into_iter());
        self
    }

    fn dump_genres(&mut self, config: &config::Parsing, genres: Genres<'_>) -> &mut Self {
        write_texts(
            self,
            config.id3v2.genres.clone(),
            genres.value.into_iter().map(|genre| genre.value.into_owned()),
        );
        self
    }

    fn dump_picture(&mut self, picture: Option<Picture<'_, '_>>) -> &mut Self {
        if let Some(picture) = picture {
            self.insert_picture(picture.into()).unwrap();
        }
        self
    }
}
