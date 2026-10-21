use std::str::FromStr;

use nghe_proc_macro::api_derive;
use uuid::Uuid;

use crate::format;

#[api_derive(serde(case = "lowercase"))]
#[derive(Default, Clone, Copy)]
pub enum Format {
    #[default]
    Raw,
    #[serde(untagged)]
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

#[cfg(test)]
#[coverage(off)]
mod tests {
    use rstest::rstest;
    use uuid::uuid;

    use super::*;

    #[rstest]
    #[case(
        "id=2613efae-e8e6-4233-bf62-43451455a1f7",
        Some(Request::builder().id(uuid!("2613efae-e8e6-4233-bf62-43451455a1f7")).build())
    )]
    #[case(
        "id=2613efae-e8e6-4233-bf62-43451455a1f7&format=raw",
        Some(Request::builder()
            .id(uuid!("2613efae-e8e6-4233-bf62-43451455a1f7"))
            .format(Format::Raw)
            .build()
        )
    )]
    #[case(
        "id=2613efae-e8e6-4233-bf62-43451455a1f7&format=raw&timeOffset=10",
        Some(Request::builder()
            .id(uuid!("2613efae-e8e6-4233-bf62-43451455a1f7"))
            .format(Format::Raw)
            .time_offset(10)
            .build()
        )
    )]
    #[case(
        "id=2613efae-e8e6-4233-bf62-43451455a1f7&format=opus",
        Some(Request::builder()
            .id(uuid!("2613efae-e8e6-4233-bf62-43451455a1f7"))
            .format(Format::Transcode(format::Transcode::Opus))
            .build()
        )
    )]
    #[case(
        "id=2613efae-e8e6-4233-bf62-43451455a1f7&maxBitRate=64&format=wav",
        Some(Request::builder()
            .id(uuid!("2613efae-e8e6-4233-bf62-43451455a1f7"))
            .max_bit_rate(64)
            .format(Format::Transcode(format::Transcode::Wav))
            .build()
        )
    )]
    #[case("id=2613efae-e8e6-4233-bf62-43451455a1f7&format=err", None)]
    #[case("format=err", None)]
    #[case("format=transcode", None)]
    #[case("format=raw&timeOffset=err", None)]
    fn test_deserialize(#[case] url: &str, #[case] request: Option<Request>) {
        crate::test_deserialize_form!(url, request, Request);
    }

    crate::test_roundtrip_request!(Request);
}
