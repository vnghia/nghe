use nghe_proc_macro::api_derive;
use uuid::Uuid;

use crate::common::format;

#[api_derive(request = false)]
#[derive(Default, Clone, Copy)]
pub enum Format {
    #[default]
    Raw,
    Transcode(format::Transcode),
}

#[api_derive]
#[endpoint(path = "stream", url_only = true)]
#[derive(Clone, Copy)]
pub struct Request {
    pub id: Uuid,
    pub max_bit_rate: Option<u32>,
    pub format: Option<Format>,
    pub time_offset: Option<u32>,
}

impl From<format::Transcode> for Format {
    fn from(value: format::Transcode) -> Self {
        Self::Transcode(value)
    }
}

mod serde {
    use ::serde::{de, Deserialize, Deserializer};

    use super::*;

    impl<'de> Deserialize<'de> for Format {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            match <&'de str>::deserialize(deserializer)? {
                "raw" => Ok(Self::Raw),
                format => {
                    Ok(Self::Transcode(format.parse().map_err(|_| {
                        de::Error::custom("Could not parse stream format parameter")
                    })?))
                }
            }
        }
    }
}
