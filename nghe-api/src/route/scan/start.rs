use nghe_proc_macro::api_derive;
use uuid::Uuid;

#[api_derive(command = true)]
#[derive(Clone, Copy, Default)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Full {
    #[serde(default)]
    pub file: bool,
    #[serde(default)]
    pub external_lyric: bool,
    #[serde(default)]
    pub dir_image: bool,
    #[serde(default)]
    pub information: bool,
}

#[api_derive]
#[endpoint(path = "startScan")]
#[cfg_attr(test, derive(Default, PartialEq))]
pub struct Request {
    pub music_folder_id: Uuid,
    #[serde(default)]
    #[conf(flatten, prefix)]
    pub full: Full,
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
        "musicFolderId=61d78c98-e9a3-43f2-bbd1-8645c037d0be",
        Some(Request {
            music_folder_id: uuid!(
                "61d78c98-e9a3-43f2-bbd1-8645c037d0be"
            ),
            ..Default::default()
        })
    )]
    #[case("musicFolderId=none", None)]
    #[case("musicFolderId=61d78c98-e9a3-43f2-bbd1-8645c037d0be&full=true", None)]
    fn test_deserialize(#[case] url: &str, #[case] request: Option<Request>) {
        assert_eq!(serde_html_form::from_str::<Request>(url).ok(), request);
    }
}
