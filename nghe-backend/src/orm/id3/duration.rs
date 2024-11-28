use std::ops::Add;

use num_traits::ToPrimitive;

use super::song;
use crate::Error;

pub trait Trait {
    fn duration(&self) -> Result<u32, Error>;
}

impl Trait for f32 {
    fn duration(&self) -> Result<u32, Error> {
        self.ceil().to_u32().ok_or_else(|| Error::CouldNotConvertFloatToInteger(*self))
    }
}

impl Trait for song::durations::Durations {
    fn duration(&self) -> Result<u32, Error> {
        self.value
            .as_ref()
            .map(|value| {
                value
                    .iter()
                    .copied()
                    .reduce(song::durations::Duration::add)
                    .ok_or_else(|| Error::DatabaseSongDurationIsEmpty)?
                    .value
                    .duration()
            })
            .transpose()
            .map(Option::unwrap_or_default)
    }
}

impl Trait for song::Song {
    fn duration(&self) -> Result<u32, Error> {
        self.property.duration.duration()
    }
}

impl Trait for Vec<song::Song> {
    fn duration(&self) -> Result<u32, Error> {
        self.iter().map(|song| song.property.duration).sum::<f32>().duration()
    }
}
