use std::fmt::Debug;

use nghe_proc_macro::api_derive;
use strum::{EnumString, IntoStaticStr};

#[derive(Debug, Clone, Copy)]
pub struct CacheControl {
    pub duration: std::time::Duration,
    pub immutable: bool,
}

pub trait Trait: Debug + Copy {
    const CACHE_CONTROL: CacheControl = CacheControl::const_default();

    fn mime(&self) -> &'static str;
    fn extension(&self) -> &'static str;
}

#[api_derive]
#[derive(Clone, Copy, IntoStaticStr, EnumString)]
#[strum(serialize_all = "lowercase")]
#[cfg_attr(feature = "test", derive(strum::AsRefStr))]
pub enum Transcode {
    Aac,
    Flac,
    Mp3,
    Opus,
    Wav,
    Wma,
}

impl CacheControl {
    pub const fn const_default() -> Self {
        Self { duration: std::time::Duration::from_days(1), immutable: false }
    }
}

impl Trait for Transcode {
    fn mime(&self) -> &'static str {
        match self {
            Self::Aac => "audio/aac",
            Self::Flac => "audio/flac",
            Self::Mp3 => "audio/mpeg",
            Self::Opus => "audio/ogg",
            Self::Wav => "audio/wav",
            Self::Wma => "audio/x-ms-wma",
        }
    }

    fn extension(&self) -> &'static str {
        self.into()
    }
}
