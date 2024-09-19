pub mod audio;
pub mod property;

use xxhash_rust::xxh3::xxh3_64;

use crate::Error;

#[derive(Debug)]
pub struct File<F> {
    pub data: Vec<u8>,
    pub property: property::File<F>,
}

impl<F> File<F> {
    pub fn new(data: Vec<u8>, format: F) -> Result<Self, Error> {
        let hash = xxh3_64(&data);
        let size = data.len().try_into()?;
        Ok(Self { data, property: property::File { hash, size, format } })
    }
}
