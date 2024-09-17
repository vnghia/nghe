#![allow(clippy::wrong_self_convention)]

use nghe_api::common::filesystem;
use typed_path::{PathType, Utf8TypedPath, Utf8TypedPathBuf, Utf8UnixPathBuf, Utf8WindowsPathBuf};

#[derive(Debug, Clone, Copy)]
pub struct Const<const ty: filesystem::Type>;

#[derive(Debug, Clone, Copy)]
pub struct Builder(pub filesystem::Type);

pub type Local = Const<{ filesystem::Type::Local }>;
pub type S3 = Const<{ filesystem::Type::S3 }>;

macro_rules! builder_from_str {
    ($ty:expr, $value:ident) => {{
        match $ty {
            filesystem::Type::Local if cfg!(windows) => {
                Utf8TypedPath::new($value, PathType::Windows)
            }
            _ => Utf8TypedPath::new($value, PathType::Unix),
        }
    }};
}

macro_rules! builder_from_string {
    ($ty:expr, $value:ident) => {{
        let $value = $value.into();
        match $ty {
            filesystem::Type::Local if cfg!(windows) => {
                Utf8TypedPathBuf::Windows(Utf8WindowsPathBuf::from($value))
            }
            _ => Utf8TypedPathBuf::Unix(Utf8UnixPathBuf::from($value)),
        }
    }};
}

#[cfg(test)]
macro_rules! builder_empty {
    ($ty:expr) => {
        match $ty {
            filesystem::Type::Local if cfg!(windows) => Utf8TypedPathBuf::new(PathType::Windows),
            _ => Utf8TypedPathBuf::new(PathType::Unix),
        }
    };
}

impl<const ty: filesystem::Type> Const<ty> {
    pub fn from_str(value: &(impl AsRef<str> + ?Sized)) -> Utf8TypedPath<'_> {
        builder_from_str!(ty, value)
    }

    pub fn from_string(value: impl Into<String>) -> Utf8TypedPathBuf {
        builder_from_string!(ty, value)
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
