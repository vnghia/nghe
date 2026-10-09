use nghe_proc_macro::api_derive;
use uuid::Uuid;

#[api_derive]
#[endpoint(path = "savePlayQueue")]
#[cfg_attr(test, derive(PartialEq))]
pub struct Request {
    #[serde(rename = "id")]
    #[conf(repeat, long)]
    pub ids: Vec<Uuid>,
    pub current: Option<Uuid>,
    pub position: Option<u64>,
}

#[api_derive]
pub struct Response;

#[cfg(test)]
#[coverage(off)]
mod tests {
    use rstest::rstest;
    use uuid::uuid;

    use super::*;

    #[rstest]
    #[case(
        "id=61d78c98-e9a3-43f2-bbd1-8645c037d0be&id=66ff5f1b-e897-42dc-88ad-7a5972a691c0",
        Some(Request {
            ids: vec![
                uuid!("61d78c98-e9a3-43f2-bbd1-8645c037d0be"),
                uuid!("66ff5f1b-e897-42dc-88ad-7a5972a691c0"),
            ],
            current: None,
            position: None,
        })
    )]
    #[case(
        "id=66ff5f1b-e897-42dc-88ad-7a5972a691c0&position=",
        Some(Request {
            ids: vec![
                uuid!("66ff5f1b-e897-42dc-88ad-7a5972a691c0"),
            ],
            current: None,
            position: None,
        })
    )]
    #[case(
        "id=66ff5f1b-e897-42dc-88ad-7a5972a691c0&current=61d78c98-e9a3-43f2-bbd1-8645c037d0be",
        Some(Request {
            ids: vec![
                uuid!("66ff5f1b-e897-42dc-88ad-7a5972a691c0"),
            ],
            current: Some(uuid!("61d78c98-e9a3-43f2-bbd1-8645c037d0be")),
            position: None,
        })
    )]
    #[case(
        "id=66ff5f1b-e897-42dc-88ad-7a5972a691c0&position=10",
        Some(Request {
            ids: vec![
                uuid!("66ff5f1b-e897-42dc-88ad-7a5972a691c0"),
            ],
            current: None,
            position: Some(10),
        })
    )]
    #[case(
        "position=10",
        Some(Request { ids: vec![], current: None, position: Some(10) })
    )]
    #[case("id=", None)]
    #[case("id=error", None)]
    #[case("id=66ff5f1b-e897-42dc-88ad-7a5972a691c0&current=", None)]
    #[case("id=66ff5f1b-e897-42dc-88ad-7a5972a691c0&current=error", None)]
    fn test_deserialize(#[case] url: &str, #[case] request: Option<Request>) {
        assert_eq!(serde_html_form::from_str::<Request>(url).ok(), request);
    }
}
