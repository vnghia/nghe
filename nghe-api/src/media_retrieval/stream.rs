use nghe_proc_macro::api_derive;
use uuid::Uuid;

use crate::common::format;

#[api_derive(request = true, serde = false)]
pub enum Format {
    Raw,
    Transcode(format::Transcode),
}

#[api_derive(endpoint = true)]
#[endpoint(path = "stream", binary = true)]
pub struct Request {
    pub id: Uuid,
    pub max_bit_rate: Option<u32>,
    pub format: Option<Format>,
    pub time_offset: Option<u32>,
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
