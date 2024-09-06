mod artist;
mod common;
mod date;
mod extract;
mod metadata;
mod position;
mod property;

#[cfg(test)]
mod dump;

use std::borrow::Cow;
use std::io::{Read, Seek};

pub use artist::{Artist, Artists};
pub use common::Common;
pub use date::Date;
#[cfg(test)]
use dump::MetadataDumper;
use enum_dispatch::enum_dispatch;
use extract::{MetadataExtractor, PropertyExtractor};
use isolang::Language;
use lofty::config::ParseOptions;
use lofty::file::{AudioFile, FileType};
use lofty::flac::FlacFile;
pub use metadata::Metadata;
pub use position::{Position, TrackDisc};
pub use property::Property;

use crate::{config, Error};

#[derive(Debug)]
#[cfg_attr(test, derive(derivative::Derivative, fake::Dummy))]
#[cfg_attr(test, derivative(PartialEq))]
pub struct Media<'a> {
    pub metadata: Metadata<'a>,
    #[cfg_attr(test, derivative(PartialEq = "ignore"))]
    pub property: Property,
}

#[enum_dispatch]
pub enum File {
    Flac(FlacFile),
}

impl File {
    pub fn read_from(
        reader: &mut (impl Read + Seek),
        parse_options: ParseOptions,
        file_type: FileType,
    ) -> Result<Self, Error> {
        match file_type {
            FileType::Flac => {
                FlacFile::read_from(reader, parse_options).map(Self::from).map_err(Error::from)
            }
            _ => Err(Error::MediaFileTypeNotSupported(file_type)),
        }
    }

    pub fn media<'a>(&'a self, config: &'a config::Parsing) -> Result<Media<'a>, Error> {
        Ok(Media { metadata: self.metadata(config)?, property: self.property()? })
    }
}

#[cfg(test)]
mod test {
    use std::io::{Cursor, Write};

    use lofty::config::WriteOptions;
    use typed_path::Utf8TypedPath;

    use super::*;

    impl File {
        pub fn save_to_bytes(&self, write_options: WriteOptions) -> Vec<u8> {
            let mut cursor = Cursor::new(vec![]);
            match self {
                File::Flac(flac_file) => {
                    flac_file.save_to(&mut cursor, write_options).unwrap();
                }
            }
            cursor.into_inner()
        }

        pub fn save_to_path(&self, path: &Utf8TypedPath<'_>, write_options: WriteOptions) {
            let bytes = self.save_to_bytes(write_options);
            std::fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(path.as_str())
                .unwrap()
                .write_all(&bytes)
                .unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use time::Month;

    use super::*;
    use crate::test::assets;

    #[rstest]
    #[case(FileType::Flac)]
    fn test_media(#[case] file_type: FileType) {
        let mut file = std::fs::File::open(assets::path(file_type).as_str()).unwrap();
        let file = File::read_from(&mut file, ParseOptions::default(), file_type).unwrap();

        let config = config::Parsing::default();
        let media = file.media(&config).unwrap();
        let metadata = media.metadata;

        let song = metadata.song;
        assert_eq!(song.name, "Sample");
        assert_eq!(song.date, Date::default());
        assert_eq!(song.release_date, Date::default());
        assert_eq!(song.original_release_date, Date::default());
        assert_eq!(song.mbz_id, None);

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

        let song = metadata.artists.song;
        assert_eq!(
            song,
            &[
                ("Artist1", uuid::uuid!("1ffedd2d-f63d-4dc2-9332-d3132e5134ac")).into(),
                "Artist2".into()
            ]
        );
        let album = metadata.artists.album;
        assert_eq!(album, &["Artist1".into(), "Artist3".into()]);

        assert_eq!(
            metadata.track_disc,
            TrackDisc {
                track: Position { number: Some(10), total: None },
                disc: Position { number: Some(5), total: Some(10) }
            }
        );

        assert_eq!(metadata.languages, &[Language::Eng, Language::Vie]);
        assert!(metadata.genres.is_empty());
        assert!(metadata.compilation);

        assert_eq!(media.property, Property::default(file_type));
    }
}
