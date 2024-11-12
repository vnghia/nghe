pub mod audio;

use std::time::Duration;

use axum_extra::headers::{CacheControl, ETag};
use nghe_api::common::format;
use typed_path::{Utf8NativePath, Utf8NativePathBuf};
use xxhash_rust::xxh3::xxh3_64;

use crate::http::binary::property;
use crate::http::header::ToETag;
use crate::Error;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(test, derive(PartialEq, Eq, fake::Dummy))]
pub struct Property<F: format::Trait> {
    pub hash: u64,
    pub size: u32,
    pub format: F,
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(test, derive(PartialEq, Eq, fake::Dummy))]
pub struct PropertySize<F: format::Trait> {
    pub size: u64,
    pub format: F,
}

#[derive(Debug)]
pub struct File<F: format::Trait> {
    pub data: Vec<u8>,
    pub property: Property<F>,
}

impl<F: format::Trait> Property<F> {
    pub fn new(data: &[u8], format: F) -> Result<Self, Error> {
        let hash = xxh3_64(data);
        let size = data.len().try_into()?;
        Ok(Self { hash, size, format })
    }

    pub fn replace<FN: format::Trait>(&self, format: FN) -> Property<FN> {
        Property { hash: self.hash, size: self.size, format }
    }

    fn path_dir(&self, base: impl AsRef<Utf8NativePath>) -> Utf8NativePathBuf {
        let hash = self.hash.to_le_bytes();

        // Avoid putting too many files in a single directory
        let first = faster_hex::hex_string(&hash[..1]);
        let second = faster_hex::hex_string(&hash[1..]);

        base.as_ref().join(first).join(second).join(self.size.to_string())
    }

    pub fn path(
        &self,
        base: impl AsRef<Utf8NativePath>,
        name: impl Into<Option<&str>>,
    ) -> Utf8NativePathBuf {
        let path = self.path_dir(base);
        if let Some(name) = name.into() {
            path.join(name).with_extension(self.format.extension())
        } else {
            path
        }
    }

    pub async fn path_create_dir(
        &self,
        base: impl AsRef<Utf8NativePath>,
        name: &str,
    ) -> Result<Utf8NativePathBuf, Error> {
        let path = self.path_dir(base);
        tokio::fs::create_dir_all(&path).await?;
        Ok(path.join(name).with_extension(self.format.extension()))
    }
}

impl<F: format::Trait> property::Trait for Property<F> {
    const SEEKABLE: bool = true;

    fn mime(&self) -> &'static str {
        self.format.mime()
    }

    fn size(&self) -> Option<u64> {
        Some(self.size.into())
    }

    fn etag(&self) -> Result<Option<ETag>, Error> {
        Some(u64::to_etag(&self.hash)).transpose()
    }

    fn cache_control() -> CacheControl {
        CacheControl::new().with_private().with_max_age(Duration::from_days(1))
    }
}

impl<F: format::Trait> property::Trait for PropertySize<F> {
    const SEEKABLE: bool = true;

    fn mime(&self) -> &'static str {
        self.format.mime()
    }

    fn size(&self) -> Option<u64> {
        Some(self.size)
    }

    fn etag(&self) -> Result<Option<ETag>, Error> {
        Ok(None)
    }

    fn cache_control() -> CacheControl {
        Property::<F>::cache_control()
    }
}

impl<F: format::Trait> File<F> {
    pub fn new(data: Vec<u8>, format: F) -> Result<Self, Error> {
        let property = Property::new(&data, format)?;
        Ok(Self { data, property })
    }
}
