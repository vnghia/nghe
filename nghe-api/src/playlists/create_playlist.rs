use nghe_proc_macro::api_derive;
use serde_with::serde_as;
use uuid::Uuid;

// TODO: Optimize this after https://github.com/serde-rs/serde/issues/1183
#[serde_as]
#[api_derive(request = true, copy = false)]
#[serde(untagged)]
pub enum CreateOrUpdate {
    Create {
        name: String,
    },
    Update {
        #[serde_as(as = "serde_with::DisplayFromStr")]
        playlist_id: Uuid,
    },
}

#[api_derive]
#[endpoint(path = "createPlaylist")]
pub struct Request {
    #[serde(flatten)]
    pub create_or_update: CreateOrUpdate,
    #[serde(rename = "songId")]
    pub song_ids: Option<Vec<Uuid>>,
}

#[api_derive]
pub struct Response {
    // TODO: Return playlist
}

#[cfg(any(test, feature = "test"))]
mod test {
    use super::*;

    impl From<String> for CreateOrUpdate {
        fn from(value: String) -> Self {
            Self::Create { name: value }
        }
    }

    impl From<Uuid> for CreateOrUpdate {
        fn from(value: Uuid) -> Self {
            Self::Update { playlist_id: value }
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use uuid::uuid;

    use super::*;

    #[rstest]
    #[case(
        "name=ef14c42b-6efa-45f3-961c-74856fd431d5",
        Some(Request {
            create_or_update: "ef14c42b-6efa-45f3-961c-74856fd431d5".to_owned().into(),
            song_ids: None,
        })
    )]
    #[case(
        "name=ef14c42b-6efa-45f3-961c-74856fd431d5&\
        songId=2b839103-04ab-4b39-9b05-8c664590eda4",
        Some(Request {
            create_or_update: "ef14c42b-6efa-45f3-961c-74856fd431d5".to_owned().into(),
            song_ids: Some(vec![uuid!("2b839103-04ab-4b39-9b05-8c664590eda4")]),
        })
    )]
    #[case(
        "playlistId=ef14c42b-6efa-45f3-961c-74856fd431d5",
        Some(Request {
            create_or_update: uuid!("ef14c42b-6efa-45f3-961c-74856fd431d5").into(),
            song_ids: None,
        })
    )]
    #[case(
        "playlistId=ef14c42b-6efa-45f3-961c-74856fd431d5&\
        songId=2b839103-04ab-4b39-9b05-8c664590eda4",
        Some(Request {
            create_or_update: uuid!("ef14c42b-6efa-45f3-961c-74856fd431d5").into(),
            song_ids: Some(vec![uuid!("2b839103-04ab-4b39-9b05-8c664590eda4")]),
        })
    )]
    #[case("playlistId=none", None)]
    fn test_deserialize(#[case] url: &str, #[case] request: Option<Request>) {
        assert_eq!(serde_html_form::from_str::<Request>(url).ok(), request);
    }
}
