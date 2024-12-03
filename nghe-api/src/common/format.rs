use std::fmt::Debug;

use nghe_proc_macro::api_derive;
use strum::{EnumString, IntoStaticStr};

pub trait Trait: Debug + Copy {
    fn mime(&self) -> &'static str;
    fn extension(&self) -> &'static str;
}

#[api_derive]
#[derive(IntoStaticStr, EnumString)]
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
