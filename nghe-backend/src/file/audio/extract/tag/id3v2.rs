use std::num::NonZeroU8;
use std::str::FromStr;

use indexmap::IndexSet;
use isolang::Language;
use lofty::id3::v2::{AttachedPictureFrame, Frame, Id3v2Tag, Id3v2Version, TimestampFrame};
use uuid::Uuid;

use crate::config::parsing::id3v2::frame;
use crate::file::audio::{Album, Artist, Artists, Date, Genres, NameDateMbz, TrackDisc, extract};
use crate::file::picture::Picture;
use crate::{Error, config, error};

fn get_text<'a>(tag: &'a Id3v2Tag, frame_id: &'a frame::Id) -> Result<Option<&'a str>, Error> {
    match frame_id {
        frame::Id::Text(frame_id) => Ok(tag.get_text(frame_id)),
        frame::Id::UserText(description) => Ok(tag.get_user_text(description)),
        frame::Id::Time(_) => error::Kind::InvalidId3v2FrameIdConfigType.into(),
    }
}

fn get_texts<'a>(
    tag: &'a Id3v2Tag,
    frame_id: &'a frame::Id,
    separator: char,
) -> Result<impl Iterator<Item = &'a str>, Error> {
    get_text(tag, frame_id).map(|text| {
        text.unwrap_or_default().split(match tag.original_version() {
            Id3v2Version::V4 => frame::Id::ID3V24_SEPARATOR,
            _ => separator,
        })
    })
}

impl Date {
    fn extract_id3v2(tag: &Id3v2Tag, frame_id: Option<&frame::Id>) -> Result<Self, Error> {
        if let Some(frame_id) = frame_id {
            match frame_id {
                frame::Id::Time(frame_id) => Ok(
                    if let Some(Frame::Timestamp(TimestampFrame { timestamp, .. })) =
                        tag.get(frame_id)
                    {
                        Self {
                            year: Some(timestamp.year.into()),
                            month: timestamp.month.map(time::Month::try_from).transpose()?,
                            day: timestamp.day.map(NonZeroU8::try_from).transpose()?,
                        }
                    } else {
                        Self::default()
                    },
                ),
                _ => get_text(tag, frame_id)?
                    .map(Self::from_str)
                    .transpose()
                    .map(Option::unwrap_or_default),
            }
        } else {
            Ok(Self::default())
        }
    }
}

impl<'a> NameDateMbz<'a> {
    fn extract_id3v2(
        tag: &'a Id3v2Tag,
        config: &'a config::parsing::id3v2::Common,
    ) -> Result<Self, Error> {
        Ok(Self {
            name: get_text(tag, &config.name)?.ok_or_else(|| error::Kind::MissingMediaName)?.into(),
            date: Date::extract_id3v2(tag, config.date.as_ref())?,
            release_date: Date::extract_id3v2(tag, config.release_date.as_ref())?,
            original_release_date: Date::extract_id3v2(tag, config.original_release_date.as_ref())?,
            mbz_id: get_text(tag, &config.mbz_id)?
                .map(|mbz_id| {
                    Uuid::from_str(mbz_id)
                        .map_err(|_| error::Kind::InvalidMbzIdTagFormat(mbz_id.to_owned()))
                })
                .transpose()?,
        })
    }
}

impl<'a> Artist<'a> {
    fn extract_id3v2(
        tag: &'a Id3v2Tag,
        config: &'a config::parsing::id3v2::Artist,
        separator: char,
    ) -> Result<IndexSet<Self>, Error> {
        let names = get_texts(tag, &config.name, separator)?;
        let mbz_ids = get_texts(tag, &config.mbz_id, separator)?;
        Self::try_collect(names, mbz_ids)
    }
}

pub trait Has<'a> {
    fn tag(&'a self) -> Result<&'a Id3v2Tag, Error>;
}

impl<'a> extract::Metadata<'a> for Id3v2Tag {
    fn song(&'a self, config: &'a config::Parsing) -> Result<NameDateMbz<'a>, Error> {
        NameDateMbz::extract_id3v2(self, &config.id3v2.song)
    }

    fn album(&'a self, config: &'a config::Parsing) -> Result<Album<'a>, Error> {
        Album::extract_id3v2(self, &config.id3v2.album)
    }

    fn artists(&'a self, config: &'a config::Parsing) -> Result<Artists<'a>, Error> {
        Artists::new(
            Artist::extract_id3v2(self, &config.id3v2.artists.song, config.id3v2.separator)?,
            Artist::extract_id3v2(self, &config.id3v2.artists.album, config.id3v2.separator)?,
            get_text(self, &config.id3v2.compilation)?.is_some_and(|s| !s.is_empty()),
        )
    }

    fn track_disc(&'a self, config: &'a config::Parsing) -> Result<TrackDisc, Error> {
        let config::parsing::id3v2::TrackDisc { track_position, disc_position } =
            &config.id3v2.track_disc;
        TrackDisc::parse(
            get_text(self, track_position)?,
            None,
            get_text(self, disc_position)?,
            None,
        )
    }

    fn languages(&'a self, config: &'a config::Parsing) -> Result<Vec<isolang::Language>, Error> {
        Ok(get_texts(self, &config.id3v2.languages, config.id3v2.separator)?
            .map(|language| {
                Language::from_str(language)
                    .map_err(|_| error::Kind::InvalidLanguageTagFormat(language.to_owned()))
            })
            .try_collect()?)
    }

    fn genres(&'a self, config: &'a config::Parsing) -> Result<Genres<'a>, Error> {
        Ok(get_texts(self, &config.id3v2.genres, config.id3v2.separator)?.collect())
    }

    fn picture(&'a self) -> Result<Option<Picture<'static, 'a>>, Error> {
        let mut iter = self.into_iter();
        iter.find_map(|frame| {
            if let Frame::Picture(AttachedPictureFrame { picture, .. }) = frame
                && (cfg!(test)
                    && picture
                        .description()
                        .is_some_and(|description| description == Picture::TEST_DESCRIPTION))
            {
                Some(picture.try_into())
            } else {
                None
            }
        })
        .transpose()
    }
}
