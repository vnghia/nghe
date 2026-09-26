use std::num::NonZeroU64;
use std::ops::Bound;

use axum_extra::headers::{ETag, Range};
use concat_string::concat_string;

use crate::{Error, error};

pub trait ToOffset {
    fn to_offset(&self, size: NonZeroU64) -> Result<u64, Error>;
}

impl ToOffset for Range {
    fn to_offset(&self, size: NonZeroU64) -> Result<u64, Error> {
        if let Bound::Included(offset) = self
            .satisfiable_ranges(size.get())
            .next()
            .ok_or_else(|| error::Kind::InvalidRangeHeader(self.to_owned()))?
            .0
        {
            Ok(offset)
        } else {
            error::Kind::InvalidRangeHeader(self.to_owned()).into()
        }
    }
}

pub trait ToETag: ToString {
    fn to_etag(&self) -> Result<ETag, Error> {
        concat_string!("\"", self.to_string(), "\"")
            .parse()
            .map_err(color_eyre::Report::from)
            .map_err(Error::from)
    }
}

impl ToETag for u64 {}
