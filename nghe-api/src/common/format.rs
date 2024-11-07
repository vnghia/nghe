use nghe_proc_macro::api_derive;
use strum::{EnumString, IntoStaticStr};

pub trait Format: Copy {
    fn mime(&self) -> &'static str;
    fn extension(&self) -> &'static str;
}

#[api_derive(request = true)]
#[derive(Clone, Copy, PartialEq, Eq, IntoStaticStr, EnumString)]
#[cfg_attr(feature = "test", derive(strum::AsRefStr))]
#[cfg_attr(feature = "test", strum(serialize_all = "lowercase"))]
pub enum Transcode {
    Aac,
    Flac,
    Mp3,
    Opus,
    Wav,
    Wma,
}

impl Format for Transcode {
    fn mime(&self) -> &'static str {
        match self {
            Self::Aac => "audio/aac",
            Self::Flac => "audio/flac",
            Self::Mp3 => "audio/mpeg",
            Self::Opus => "audio/opus",
            Self::Wav => "audio/wav",
            Self::Wma => "audio/x-ms-wma",
        }
    }

    fn extension(&self) -> &'static str {
        self.into()
    }
}
