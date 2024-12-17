pub mod album;
pub mod artist;
pub mod date;
pub mod genre;
pub mod song;

pub mod builder {
    pub mod artist {
        pub use super::super::artist::ArtistBuilder as Builder;
        pub use super::super::artist::artist_builder::*;
    }

    pub mod album {
        pub use super::super::album::AlbumBuilder as Builder;
        pub use super::super::album::album_builder::*;
    }

    pub mod song {
        pub use super::super::song::SongBuilder as Builder;
        pub use super::super::song::song_builder::*;
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use nghe_proc_macro::api_derive;
    use rstest::rstest;
    use serde_json::json;

    #[api_derive]
    pub struct Test {
        duration: time::Duration,
    }

    #[rstest]
    #[case(time::Duration::seconds_f32(1.5), 2)]
    #[case(time::Duration::seconds_f32(2.1), 3)]
    #[case(time::Duration::seconds_f32(10.0), 10)]
    fn test_serialize_duration(#[case] duration: time::Duration, #[case] result: i64) {
        assert_eq!(
            serde_json::to_string(&Test { duration }).unwrap(),
            serde_json::to_string(&json!({
                "duration": result,
            }))
            .unwrap()
        );
    }
}
