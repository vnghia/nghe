use std::ops::Bound;

use axum_extra::headers::Range;

use crate::Error;

pub fn from_range(range: Option<Range>, size: u64) -> Result<Option<u64>, Error> {
    range
        .map(|range| {
            if let Bound::Included(offset) =
                range.satisfiable_ranges(size).next().ok_or_else(|| Error::InvalidRangeHeader)?.0
            {
                Ok(offset)
            } else {
                Err(Error::InvalidRangeHeader)
            }
        })
        .transpose()
}
