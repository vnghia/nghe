pub mod audio;

use nghe_api::common::format;
use typed_path::{Utf8NativePath, Utf8NativePathBuf};
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

    pub fn replace<FN: format::Format>(&self, format: FN) -> Property<FN> {
        Property { hash: self.hash, size: self.size, format }
    }

    pub fn path(
        &self,
        base: impl AsRef<Utf8NativePath>,
        name: impl Into<Option<&'static str>>,
    ) -> Utf8NativePathBuf {
        let hash = self.hash.to_le_bytes();

        // Avoid putting too many files in a single directory
        let first = faster_hex::hex_string(&hash[..1]);
        let second = faster_hex::hex_string(&hash[1..]);

        let path = base.as_ref().join(first).join(second).join(self.size.to_string());
        if let Some(name) = name.into() {
            path.join(name).with_extension(self.format.extension())
        } else {
            path
        }
    }
}

impl<F: format::Format> File<F> {
    pub fn new(data: Vec<u8>, format: F) -> Result<Self, Error> {
        let property = Property::new(&data, format)?;
        Ok(Self { data, property })
    }
}
