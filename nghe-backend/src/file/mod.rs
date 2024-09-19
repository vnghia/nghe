pub mod audio;

use xxhash_rust::xxh3::xxh3_64;

use crate::Error;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(test, derive(PartialEq, Eq, fake::Dummy))]
pub struct Property<F> {
    pub hash: u64,
    pub size: u32,
    pub format: F,
}

#[derive(Debug)]
pub struct File<F> {
    pub data: Vec<u8>,
    pub property: Property<F>,
}

impl<F> File<F> {
    pub fn new(data: Vec<u8>, format: F) -> Result<Self, Error> {
        let hash = xxh3_64(&data);
        let size = data.len().try_into()?;
        Ok(Self { data, property: Property { hash, size, format } })
    }
}
