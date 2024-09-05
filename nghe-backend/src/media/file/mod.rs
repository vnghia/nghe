mod artist;
mod common;
mod date;
mod extract;
mod metadata;
mod position;
mod property;

use std::io::{Read, Seek};

pub use artist::{Artist, Artists};
pub use common::Common;
pub use date::Date;
use enum_dispatch::enum_dispatch;
use extract::{MetadataTrait, PropertyTrait};
use isolang::Language;
use lofty::config::ParseOptions;
use lofty::file::{AudioFile, FileType};
use lofty::flac::FlacFile;
pub use metadata::Metadata;
pub use position::{Position, TrackDisc};
pub use property::Property;

use crate::{config, Error};

#[derive(Debug)]
pub struct Media<'a> {
    pub metadata: Metadata<'a>,
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
