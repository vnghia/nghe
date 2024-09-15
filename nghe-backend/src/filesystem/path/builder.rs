#![allow(clippy::wrong_self_convention)]

use nghe_api::common::filesystem;
use typed_path::{PathType, Utf8TypedPath, Utf8TypedPathBuf, Utf8UnixPathBuf, Utf8WindowsPathBuf};

#[derive(Debug, Clone, Copy)]
pub struct Const<const FILESYSTEM_TYPE: filesystem::Type>;

#[derive(Debug, Clone, Copy)]
pub struct Builder(pub filesystem::Type);

pub type Local = Const<{ filesystem::Type::Local }>;
pub type S3 = Const<{ filesystem::Type::S3 }>;

macro_rules! builder_from_str {
    ($filesystem_type:expr, $value:ident) => {{
        match $filesystem_type {
            filesystem::Type::Local if cfg!(windows) => {
                Utf8TypedPath::new($value, PathType::Windows)
            }
            _ => Utf8TypedPath::new($value, PathType::Unix),
        }
    }};
}

macro_rules! builder_from_string {
    ($filesystem_type:expr, $value:ident) => {{
        let $value = $value.into();
        match $filesystem_type {
            filesystem::Type::Local if cfg!(windows) => {
                Utf8TypedPathBuf::Windows(Utf8WindowsPathBuf::from($value))
            }
            _ => Utf8TypedPathBuf::Unix(Utf8UnixPathBuf::from($value)),
        }
    }};
}

#[cfg(test)]
macro_rules! builder_empty {
    ($filesystem_type:expr) => {
        match $filesystem_type {
            filesystem::Type::Local if cfg!(windows) => {
                // TODO: use `new` after https://github.com/chipsenkbeil/typed-path/pull/30
                Utf8TypedPathBuf::Windows(typed_path::Utf8WindowsPathBuf::new())
            }
            _ => Utf8TypedPathBuf::new(PathType::Unix),
        }
    };
}

impl<const FILESYSTEM_TYPE: filesystem::Type> Const<FILESYSTEM_TYPE> {
    pub fn from_str(value: &(impl AsRef<str> + ?Sized)) -> Utf8TypedPath<'_> {
        builder_from_str!(FILESYSTEM_TYPE, value)
    }

    pub fn from_string(value: impl Into<String>) -> Utf8TypedPathBuf {
        builder_from_string!(FILESYSTEM_TYPE, value)
    }
}

impl Builder {
    pub fn from_str(self, value: &(impl AsRef<str> + ?Sized)) -> Utf8TypedPath<'_> {
        builder_from_str!(self.0, value)
    }

    pub fn from_string(self, value: impl Into<String>) -> Utf8TypedPathBuf {
        builder_from_string!(self.0, value)
    }

    #[cfg(test)]
    pub fn empty(self) -> Utf8TypedPathBuf {
        builder_empty!(self.0)
    }
}
