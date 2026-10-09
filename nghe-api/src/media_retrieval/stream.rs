use std::str::FromStr;

use nghe_proc_macro::api_derive;
use uuid::Uuid;

use crate::common::format;

#[api_derive(request = false)]
#[derive(Default, Clone, Copy)]
#[cfg_attr(test, derive(PartialEq))]
pub enum Format {
    #[default]
    Raw,
    Transcode(format::Transcode),
}

#[api_derive]
#[endpoint(path = "stream", url_only = true)]
#[derive(Clone, Copy)]
#[cfg_attr(test, derive(Default, PartialEq))]
pub struct Request {
    pub id: Uuid,
    pub max_bit_rate: Option<u32>,
    pub format: Option<Format>,
    pub time_offset: Option<u32>,
}

impl FromStr for Format {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "raw" => Ok(Self::Raw),
            _ => s.parse::<format::Transcode>().map(Into::into),
        }
    }
}

impl From<format::Transcode> for Format {
    fn from(value: format::Transcode) -> Self {
        Self::Transcode(value)
    }
}

mod serde {
    use ::serde::{Deserialize, Deserializer, de};

    use super::*;

    impl<'de> Deserialize<'de> for Format {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let format = <&'de str>::deserialize(deserializer)?;
            format.parse().map_err(|_| de::Error::custom("Could not parse stream format parameter"))
        }
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use rstest::rstest;
    use uuid::uuid;

    use super::*;

    #[rstest]
    #[case(
        "id=2613efae-e8e6-4233-bf62-43451455a1f7",
        Some(Request {
            id: uuid!("2613efae-e8e6-4233-bf62-43451455a1f7"),
            ..Default::default()
        })
    )]
    #[case(
        "id=2613efae-e8e6-4233-bf62-43451455a1f7&format=raw",
        Some(Request {
            id: uuid!("2613efae-e8e6-4233-bf62-43451455a1f7"),
            format: Some(Format::Raw),
            ..Default::default()
        })
    )]
    #[case(
        "id=2613efae-e8e6-4233-bf62-43451455a1f7&format=raw&timeOffset=10",
        Some(Request {
            id: uuid!("2613efae-e8e6-4233-bf62-43451455a1f7"),
            format: Some(Format::Raw),
            time_offset: Some(10),
            ..Default::default()
        })
    )]
    #[case(
        "id=2613efae-e8e6-4233-bf62-43451455a1f7&format=opus",
        Some(Request {
            id: uuid!("2613efae-e8e6-4233-bf62-43451455a1f7"),
            format: Some(format::Transcode::Opus.into()),
            ..Default::default()
        })
    )]
    #[case(
        "id=2613efae-e8e6-4233-bf62-43451455a1f7&maxBitRate=64&format=wav",
        Some(Request {
            id: uuid!("2613efae-e8e6-4233-bf62-43451455a1f7"),
            max_bit_rate: Some(64),
            format: Some(format::Transcode::Wav.into()),
            ..Default::default()
        })
    )]
    #[case("id=2613efae-e8e6-4233-bf62-43451455a1f7&format=err", None)]
    #[case("format=err", None)]
    #[case("format=transcode", None)]
    #[case("format=raw&timeOffset=err", None)]
    fn test_deserialize(#[case] url: &str, #[case] request: Option<Request>) {
        assert_eq!(serde_html_form::from_str::<Request>(url).ok(), request);
    }
}
