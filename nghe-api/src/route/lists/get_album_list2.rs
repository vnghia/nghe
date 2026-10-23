#[cfg(feature = "cli")]
use conf::Conf;
use nghe_proc_macro::api_derive;
use uuid::Uuid;

use crate::id3;

// TODO: Remove this after https://github.com/cbeck88/conf-rs/pull/42
#[api_derive(cli = true, serde(sas = true))]
pub struct ByYear {
    // TODO: Optimize this after https://github.com/serde-rs/serde/issues/1183
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub from_year: u16,
    // TODO: Optimize this after https://github.com/serde-rs/serde/issues/1183
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub to_year: u16,
}

#[api_derive(cli = true)]
#[serde(tag = "type")]
pub enum Type {
    Random,
    Newest,
    Frequent,
    Recent,
    AlphabeticalByName,
    ByYear(ByYear),
    ByGenre { genre: String },
}

#[api_derive]
#[endpoint(path = "getAlbumList2")]
pub struct Request {
    #[serde(flatten, rename = "type")]
    #[conf(subcommands)]
    pub ty: Type,
    pub size: Option<u32>,
    pub offset: Option<u32>,
    #[serde(rename = "musicFolderId")]
    #[conf(repeat, long)]
    pub music_folder_ids: Vec<Uuid>,
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
#[coverage(off)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("type=random", Some(Request::builder().ty(Type::Random).build()))]
    #[case(
        "type=random&size=10",
        Some(Request::builder().ty(Type::Random).size(10).build())
    )]
    #[case("type=newest", Some(Request::builder().ty(Type::Newest).build()))]
    #[case("type=frequent", Some(Request::builder().ty(Type::Frequent).build()))]
    #[case("type=recent", Some(Request::builder().ty(Type::Recent).build()))]
    #[case(
        "type=alphabeticalByName",
        Some(Request::builder().ty(Type::AlphabeticalByName).build())
    )]
    #[case(
        "type=byYear&fromYear=1000&toYear=2000",
        Some(Request::builder()
            .ty(Type::ByYear(ByYear{
                from_year: 1000, to_year: 2000
            }))
            .build()
        )
    )]
    #[case(
        "type=byYear&fromYear=1000&toYear=2000&size=10",
        Some(Request::builder()
            .ty(Type::ByYear(ByYear{
                from_year: 1000, to_year: 2000
            }))
            .size(10)
            .build()
        )
    )]
    #[case(
        "type=byGenre&genre=Test",
        Some(Request::builder().ty(Type::ByGenre { genre: "Test".into() }).build())
    )]
    #[case(
        "type=byGenre&genre=Test&size=10",
        Some(Request::builder().ty(Type::ByGenre { genre: "Test".into() }).size(10).build())

    )]
    #[case("type=byYear&toYear=2000", None)]
    #[case("type=byYear&fromYear=From&toYear=2000", None)]
    #[case("type=byGenre", None)]
    fn test_deserialize(#[case] url: &str, #[case] request: Option<Request>) {
        crate::test_deserialize_form!(url, request, Request);
    }

    crate::test_roundtrip!(Request, Response);
}
