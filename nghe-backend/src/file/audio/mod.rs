mod artist;
mod date;
mod extract;
mod metadata;
mod name_date_mbz;
pub mod position;
mod property;

use std::io::Cursor;

pub use artist::{Artist, Artists};
pub use date::Date;
use diesel::sql_types::Text;
use diesel::{AsExpression, FromSqlRow};
use extract::{Metadata as _, Property as _};
use lofty::config::ParseOptions;
use lofty::file::AudioFile;
use lofty::flac::FlacFile;
pub use metadata::{Metadata, Song};
pub use name_date_mbz::NameDateMbz;
pub use position::TrackDisc;
pub use property::Property;
use strum::{AsRefStr, EnumString};
use xxhash_rust::xxh3::xxh3_64;

use crate::{config, Error};

#[derive(Debug)]
#[cfg_attr(test, derive(derivative::Derivative, fake::Dummy, Clone))]
#[cfg_attr(test, derivative(PartialEq))]
pub struct Audio<'a> {
    pub metadata: Metadata<'a>,
    #[cfg_attr(test, derivative(PartialEq = "ignore"))]
    pub property: Property,
    pub file: super::property::File<Format>,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    EnumString,
    AsRefStr,
    AsExpression,
    FromSqlRow,
)]
#[diesel(sql_type = Text)]
#[strum(serialize_all = "snake_case")]
#[cfg_attr(test, derive(fake::Dummy))]
pub enum Format {
    Flac,
}

pub enum File {
    Flac { format: Format, file: FlacFile, data: Vec<u8> },
}

impl File {
    pub fn read_from(
        data: Vec<u8>,
        parse_options: ParseOptions,
        format: Format,
    ) -> Result<Self, Error> {
        let mut reader = Cursor::new(&data);
        match format {
            Format::Flac => Ok(Self::Flac {
                format,
                file: FlacFile::read_from(&mut reader, parse_options)?,
                data,
            }),
        }
    }

    fn data_format(&self) -> (&[u8], Format) {
        match self {
            File::Flac { data, format, .. } => (data, *format),
        }
    }

    pub fn file_property(&self) -> Result<super::property::File<Format>, Error> {
        let (data, format) = self.data_format();
        Ok(super::property::File { hash: xxh3_64(data), size: data.len().try_into()?, format })
    }

    pub fn audio<'a>(&'a self, config: &'a config::Parsing) -> Result<Audio<'a>, Error> {
        Ok(Audio {
            metadata: self.metadata(config)?,
            property: self.property()?,
            file: self.file_property()?,
        })
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use lofty::config::WriteOptions;
    use lofty::ogg::VorbisComments;

    use super::*;

    impl File {
        pub fn clear(&mut self) -> &mut Self {
            match self {
                File::Flac { file, .. } => {
                    file.remove_id3v2();
                    file.set_vorbis_comments(VorbisComments::default());
                }
            }
            self
        }

        pub fn save_to(&self, cursor: &mut Cursor<Vec<u8>>, write_options: WriteOptions) {
            match self {
                File::Flac { file, .. } => {
                    file.save_to(cursor, write_options).unwrap();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};
    use isolang::Language;
    use nghe_api::common::filesystem;
    use position::Position;
    use rstest::rstest;
    use time::Month;

    use super::*;
    use crate::test::{assets, mock, Mock};

    #[rstest]
    fn test_media(#[values(Format::Flac)] format: Format) {
        let file = File::read_from(
            std::fs::read(assets::path(format).as_str()).unwrap(),
            ParseOptions::default(),
            format,
        )
        .unwrap();

        let config = config::Parsing::test();
        let media = file.audio(&config).unwrap();
        let metadata = media.metadata;

        let song = metadata.song;
        let main = song.main;
        assert_eq!(main.name, "Sample");
        assert_eq!(main.date, Date::default());
        assert_eq!(main.release_date, Date::default());
        assert_eq!(main.original_release_date, Date::default());
        assert_eq!(main.mbz_id, None);

        assert_eq!(
            song.track_disc,
            TrackDisc {
                track: Position { number: Some(10), total: None },
                disc: Position { number: Some(5), total: Some(10) }
            }
        );

        assert_eq!(song.languages, &[Language::Eng, Language::Vie]);

        let album = metadata.album;
        assert_eq!(album.name, "Album");
        assert_eq!(
            album.date,
            Date {
                year: Some(2000),
                month: Some(Month::December),
                day: Some(31.try_into().unwrap())
            }
        );
        assert_eq!(album.release_date, Date::default());
        assert_eq!(
            album.original_release_date,
            Date { year: Some(3000), month: Some(Month::January), day: None }
        );
        assert_eq!(album.mbz_id, None);

        let artists = metadata.artists;
        let song = artists.song;
        assert_eq!(
            song,
            &[
                ("Artist1", uuid::uuid!("1ffedd2d-f63d-4dc2-9332-d3132e5134ac")).into(),
                "Artist2".into()
            ]
        );
        let album = artists.album;
        assert_eq!(album, &["Artist1".into(), "Artist3".into()]);
        assert!(artists.compilation);

        assert!(metadata.genres.is_empty());

        assert_eq!(media.property, Property::default(format));
    }

    #[rstest]
    #[tokio::test]
    async fn test_roundtrip(
        #[future(awt)]
        #[with(0, 0)]
        mock: Mock,
        #[values(filesystem::Type::Local, filesystem::Type::S3)] ty: filesystem::Type,
        #[values(Format::Flac)] format: Format,
    ) {
        mock.add_music_folder().ty(ty).call().await;
        let music_folder = mock.music_folder(0).await;
        let audio: Audio = Faker.fake();
        let roundtrip_file = music_folder
            .add_audio()
            .path("test".into())
            .format(format)
            .audio(audio.clone())
            .call()
            .await
            .file("test".into(), format)
            .await;
        let roundtrip_media = roundtrip_file.audio(&mock.config.parsing).unwrap();
        assert_eq!(roundtrip_media.metadata.artists.song, audio.metadata.artists.song);
    }
}
