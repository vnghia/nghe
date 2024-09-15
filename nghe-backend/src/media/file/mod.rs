mod artist;
mod date;
mod extract;
mod metadata;
mod name_date_mbz;
mod position;
mod property;

use std::io::{Read, Seek};

pub use artist::{Artist, Artists};
pub use date::Date;
use extract::{Metadata as _, Property as _};
use lofty::config::ParseOptions;
use lofty::file::AudioFile;
use lofty::flac::FlacFile;
pub use metadata::{Metadata, Song};
pub use name_date_mbz::NameDateMbz;
pub use position::{Position, TrackDisc};
pub use property::Property;
use strum::{AsRefStr, EnumString};

use crate::{config, Error};

#[derive(Debug)]
#[cfg_attr(test, derive(derivative::Derivative, fake::Dummy, Clone))]
#[cfg_attr(test, derivative(PartialEq))]
pub struct Media<'a> {
    pub metadata: Metadata<'a>,
    #[cfg_attr(test, derivative(PartialEq = "ignore"))]
    pub property: Property,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, EnumString, AsRefStr)]
#[strum(serialize_all = "snake_case")]
#[cfg_attr(test, derive(fake::Dummy))]
pub enum Type {
    Flac,
}

pub enum File {
    Flac { file_type: Type, file: FlacFile },
}

impl File {
    pub fn read_from(
        reader: &mut (impl Read + Seek),
        parse_options: ParseOptions,
        file_type: Type,
    ) -> Result<Self, Error> {
        match file_type {
            Type::Flac => {
                Ok(Self::Flac { file_type, file: FlacFile::read_from(reader, parse_options)? })
            }
        }
    }

    pub fn media<'a>(&'a self, config: &'a config::Parsing) -> Result<Media<'a>, Error> {
        Ok(Media { metadata: self.metadata(config)?, property: self.property()? })
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
    use rstest::rstest;
    use time::Month;

    use super::*;
    use crate::test::{assets, mock, Mock};

    #[rstest]
    fn test_media(#[values(Type::Flac)] file_type: Type) {
        let mut file = std::fs::File::open(assets::path(file_type).as_str()).unwrap();
        let file = File::read_from(&mut file, ParseOptions::default(), file_type).unwrap();

        let config = config::Parsing::test();
        let media = file.media(&config).unwrap();
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

        assert_eq!(media.property, Property::default(file_type));
    }

    #[rstest]
    #[tokio::test]
    async fn test_roundtrip(
        #[future(awt)]
        #[with(0, 0)]
        mock: Mock,
        #[values(filesystem::Type::Local, filesystem::Type::S3)] filesystem_type: filesystem::Type,
        #[values(Type::Flac)] file_type: Type,
    ) {
        mock.add_music_folder().filesystem_type(filesystem_type).call().await;
        let music_folder = mock.music_folder(0).await;
        let media: Media = Faker.fake();
        let roundtrip_file = music_folder
            .add_media()
            .path("test".into())
            .file_type(file_type)
            .media(media.clone())
            .call()
            .await
            .file("test".into(), file_type)
            .await;
        let roundtrip_media = roundtrip_file.media(&mock.config.parsing).unwrap();
        assert_eq!(roundtrip_media.metadata.artists.song, media.metadata.artists.song);
    }
}
