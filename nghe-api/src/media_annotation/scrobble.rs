use nghe_proc_macro::api_derive;
use serde_with::{serde_as, TimestampMilliSeconds};
use time::OffsetDateTime;
use uuid::Uuid;

#[api_derive(serde_as = true)]
#[endpoint(path = "scrobble")]
#[cfg_attr(test, derive(Default, PartialEq))]
pub struct Request {
    #[serde(rename = "id")]
    pub ids: Vec<Uuid>,
    #[serde(rename = "time")]
    #[serde_as(as = "Option<Vec<TimestampMilliSeconds<i64>>>")]
    pub times: Option<Vec<OffsetDateTime>>,
    pub submission: Option<bool>,
}

#[api_derive]
pub struct Response;

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use time::macros::datetime;
    use uuid::uuid;

    use super::*;

    #[rstest]
    #[case(
        "id=d4ea6896-a838-446c-ace4-d9d13d336391",
        Some(Request {
            ids: vec![uuid!("d4ea6896-a838-446c-ace4-d9d13d336391")],
            ..Default::default()
        })
    )]
    #[case(
        "id=d4ea6896-a838-446c-ace4-d9d13d336391&\
        time=1000000000000",
        Some(Request {
            ids: vec![uuid!("d4ea6896-a838-446c-ace4-d9d13d336391")],
            times: Some(vec![datetime!(2001-09-09 01:46:40.000 UTC)]),
            ..Default::default()
        })
    )]
    fn test_deserialize(#[case] url: &str, #[case] request: Option<Request>) {
        serde_html_form::from_str::<Request>(url).unwrap();
        assert_eq!(serde_html_form::from_str::<Request>(url).ok(), request);
    }
}
