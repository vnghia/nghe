use nghe_proc_macro::api_derive;
use serde_with::TimestampMilliSeconds;
use time::OffsetDateTime;
use uuid::Uuid;

#[api_derive(serde_as = true)]
#[endpoint(path = "scrobble")]
pub struct Request {
    #[serde(rename = "id")]
    #[conf(repeat, long)]
    pub ids: Vec<Uuid>,
    #[serde(rename = "time")]
    #[serde_as(as = "Vec<TimestampMilliSeconds<i64>>")]
    #[conf(repeat, long, value_parser = crate::time::parsing::offset_date_time::from_str)]
    pub times: Vec<OffsetDateTime>,
    pub submission: Option<bool>,
}

#[api_derive]
pub struct Response;

#[cfg(test)]
#[coverage(off)]
mod tests {
    use rstest::rstest;
    use time::macros::datetime;
    use uuid::uuid;

    use super::*;

    #[rstest]
    #[case(
        "id=d4ea6896-a838-446c-ace4-d9d13d336391",
        Some(Request::builder().ids(vec![uuid!("d4ea6896-a838-446c-ace4-d9d13d336391")]).build())
    )]
    #[case(
        "id=d4ea6896-a838-446c-ace4-d9d13d336391&\
        time=1000000000000",
        Some(Request::builder()
            .ids(vec![uuid!("d4ea6896-a838-446c-ace4-d9d13d336391")])
            .times(vec![datetime!(2001-09-09 01:46:40.000 UTC)])
            .build()
        )
    )]
    fn test_deserialize(#[case] url: &str, #[case] request: Option<Request>) {
        crate::test_deserialize_form!(url, request, Request);
    }
}
