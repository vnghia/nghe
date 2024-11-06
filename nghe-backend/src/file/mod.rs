pub mod audio;

use nghe_api::common::format;
use xxhash_rust::xxh3::xxh3_64;

use crate::Error;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(test, derive(PartialEq, Eq, fake::Dummy))]
pub struct Property<F: format::Format> {
    pub hash: u64,
    pub size: u32,
    pub format: F,
}

#[derive(Debug)]
pub struct File<F: format::Format> {
    pub data: Vec<u8>,
    pub property: Property<F>,
}

impl<F: format::Format> Property<F> {
    pub fn new(data: &[u8], format: F) -> Result<Self, Error> {
        let hash = xxh3_64(data);
        let size = data.len().try_into()?;
        Ok(Self { hash, size, format })
    }

    pub fn signed_hash(&self) -> i64 {
        self.hash as _
    }

    pub fn signed_size(&self) -> i32 {
        self.size as _
    }
}

impl<F: format::Format> File<F> {
    pub fn new(data: Vec<u8>, format: F) -> Result<Self, Error> {
        let property = Property::new(&data, format)?;
        Ok(Self { data, property })
    }
}
