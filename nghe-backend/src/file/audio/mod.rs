mod artist;
mod date;
mod extract;
mod genre;
mod information;
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
pub use genre::Genres;
pub use information::Information;
use lofty::config::ParseOptions;
use lofty::file::AudioFile;
use lofty::flac::FlacFile;
pub use metadata::{Metadata, Song};
pub use name_date_mbz::NameDateMbz;
pub use position::TrackDisc;
pub use property::Property;
use strum::{AsRefStr, EnumString};

use crate::{config, Error};

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
    Flac { audio: FlacFile, file: super::File<Format> },
}

impl super::File<Format> {
    pub fn audio(self, parse_options: ParseOptions) -> Result<File, Error> {
        let mut reader = Cursor::new(&self.data);
        match self.property.format {
            Format::Flac => Ok(File::Flac {
                audio: FlacFile::read_from(&mut reader, parse_options)?,
                file: self,
            }),
        }
    }
}

impl File {
    pub fn file(&self) -> &super::File<Format> {
        match self {
            Self::Flac { file, .. } => file,
        }
    }

    pub fn extract<'a>(&'a self, config: &'a config::Parsing) -> Result<Information<'a>, Error> {
        Ok(Information {
            metadata: self.metadata(config)?,
            property: self.property()?,
            file: self.file().property,
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
                File::Flac { audio, .. } => {
                    audio.remove_id3v2();
                    audio.set_vorbis_comments(VorbisComments::default());
                }
            }
            self
        }

        pub fn save_to(&self, cursor: &mut Cursor<Vec<u8>>, write_options: WriteOptions) {
            match self {
                File::Flac { audio, .. } => {
                    audio.save_to(cursor, write_options).unwrap();
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
    use crate::file::File;
    use crate::test::{assets, mock, Mock};

    #[rstest]
    fn test_media(#[values(Format::Flac)] format: Format) {
        let file = File::new(std::fs::read(assets::path(format).as_str()).unwrap(), format)
            .unwrap()
            .audio(ParseOptions::default())
            .unwrap();

        let config = config::Parsing::test();
        let media = file.extract(&config).unwrap();
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

        assert!(metadata.genres.value.is_empty());

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
        let metadata: Metadata = Faker.fake();
        let roundtrip_file = music_folder
            .add_audio()
            .path("test".into())
            .format(format)
            .metadata(metadata.clone())
            .call()
            .await
            .file("test".into(), format)
            .await;
        let roundtrip_audio = roundtrip_file.extract(&mock.config.parsing).unwrap();
        assert_eq!(roundtrip_audio.metadata, metadata);
        assert_eq!(roundtrip_audio.property, Property::default(format));
    }
}
