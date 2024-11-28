use nghe_proc_macro::api_derive;
use serde_with::serde_as;
use uuid::Uuid;

use crate::id3;

// TODO: Optimize this after https://github.com/serde-rs/serde/issues/1183
#[serde_as]
#[api_derive(copy = false)]
#[serde(tag = "type")]
#[cfg_attr(test, derive(Default))]
pub enum Type {
    #[cfg_attr(test, default)]
    Random,
    Newest,
    Frequent,
    Recent,
    AlphabeticalByName,
    ByYear {
        #[serde_as(as = "serde_with::DisplayFromStr")]
        from_year: u16,
        #[serde_as(as = "serde_with::DisplayFromStr")]
        to_year: u16,
    },
    ByGenre {
        genre: String,
    },
}

#[api_derive]
#[endpoint(path = "getAlbumList2")]
#[cfg_attr(test, derive(Default))]
pub struct Request {
    #[serde(flatten, rename = "type")]
    pub ty: Type,
    pub size: Option<u32>,
    pub offset: Option<u32>,
    #[serde(rename = "musicFolderId")]
    pub music_folder_ids: Option<Vec<Uuid>>,
}

#[api_derive]
pub struct AlbumList2 {
    pub album: Vec<id3::album::Album>,
}

#[api_derive]
pub struct Response {
    pub album_list2: AlbumList2,
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("type=random", Some(Request { ty: Type::Random, ..Default::default() }))]
    #[case(
        "type=random&size=10",
        Some(Request { ty: Type::Random, size: Some(10), ..Default::default() })
    )]
    #[case("type=newest", Some(Request { ty: Type::Newest, ..Default::default() }))]
    #[case("type=frequent", Some(Request { ty: Type::Frequent, ..Default::default() }))]
    #[case("type=recent", Some(Request { ty: Type::Recent, ..Default::default() }))]
    #[case(
        "type=alphabeticalByName",
        Some(Request { ty: Type::AlphabeticalByName, ..Default::default() })
    )]
    #[case(
        "type=byYear&fromYear=1000&toYear=2000",
        Some(Request {
            ty: Type::ByYear {
                from_year: 1000, to_year: 2000
            }, size: None, ..Default::default()
        })
    )]
    #[case(
        "type=byYear&fromYear=1000&toYear=2000&size=10",
        Some(Request {
            ty: Type::ByYear {
                from_year: 1000, to_year: 2000
            }, size: Some(10), ..Default::default()
        })
    )]
    #[case(
        "type=byGenre&genre=Test",
        Some(Request {
            ty: Type::ByGenre { genre: "Test".to_owned() }, size: None, ..Default::default()
        })
    )]
    #[case(
        "type=byGenre&genre=Test&size=10",
        Some(Request {
            ty: Type::ByGenre { genre: "Test".to_owned() }, size: Some(10), ..Default::default()
        })
    )]
    #[case("type=byYear&toYear=2000", None)]
    #[case("type=byYear&fromYear=From&toYear=2000", None)]
    #[case("type=byGenre", None)]
    fn test_deserialize(#[case] url: &str, #[case] request: Option<Request>) {
        assert_eq!(serde_html_form::from_str::<Request>(url).ok(), request);
    }
}
