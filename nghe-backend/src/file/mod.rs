pub mod audio;

use xxhash_rust::xxh3::xxh3_64;

use crate::Error;

pub trait Mime: Copy {
    fn mime(self) -> &'static str;
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(test, derive(PartialEq, Eq, fake::Dummy))]
pub struct Property<F: Mime> {
    pub hash: u64,
    pub size: u32,
    pub format: F,
}

#[derive(Debug)]
pub struct File<F: Mime> {
    pub data: Vec<u8>,
    pub property: Property<F>,
}

impl<F: Mime> Property<F> {
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

    pub fn mime(&self) -> &'static str {
        self.format.mime()
    }
}

impl<F: Mime> File<F> {
    pub fn new(data: Vec<u8>, format: F) -> Result<Self, Error> {
        let property = Property::new(&data, format)?;
        Ok(Self { data, property })
    }
}
