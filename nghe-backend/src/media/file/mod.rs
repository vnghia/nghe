mod artist;
mod common;
mod date;
mod extract;
mod metadata;
mod position;
mod property;

pub use artist::{Artist, Artists};
pub use common::Common;
pub use date::Date;
pub use metadata::Metadata;
pub use position::{Position, TrackDisc};
pub use property::Property;

#[derive(Debug)]
pub struct Media<'a> {
    pub metadata: Metadata<'a>,
    pub property: Property,
}
