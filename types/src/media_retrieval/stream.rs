use nghe_proc_macros::{add_common_convert, add_types_derive};
use strum::AsRefStr;
use uuid::Uuid;

#[add_types_derive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, AsRefStr)]
#[strum(serialize_all = "lowercase")]
#[cfg_attr(feature = "test", derive(strum::EnumIter))]
pub enum Format {
    Raw,
    Aac,
    Flac,
    Mp3,
    Opus,
    Wav,
    Wma,
}

#[add_common_convert]
#[derive(Debug)]
pub struct StreamParams {
    pub id: Uuid,
    pub max_bit_rate: Option<u32>,
    pub format: Option<Format>,
    pub time_offset: Option<u32>,
}
