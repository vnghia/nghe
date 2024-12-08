use educe::Educe;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Common {
    pub name: String,
    #[serde_as(as = "serde_with::NoneAsEmptyString")]
    pub date: Option<String>,
    #[serde_as(as = "serde_with::NoneAsEmptyString")]
    pub release_date: Option<String>,
    #[serde_as(as = "serde_with::NoneAsEmptyString")]
    pub original_release_date: Option<String>,
    pub mbz_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artist {
    pub name: String,
    pub mbz_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Educe)]
#[educe(Default)]
pub struct Artists {
    #[educe(Default(expression = Artist::default_song()))]
    pub song: Artist,
    #[educe(Default(expression = Artist::default_album()))]
    pub album: Artist,
}

#[derive(Debug, Clone, Serialize, Deserialize, Educe)]
#[educe(Default)]
pub struct TrackDisc {
    #[educe(Default(expression = "TRACKNUMBER".into()))]
    pub track_number: String,
    #[educe(Default(expression = "TRACKTOTAL".into()))]
    pub track_total: String,
    #[educe(Default(expression = "DISCNUMBER".into()))]
    pub disc_number: String,
    #[educe(Default(expression = "DISCTOTAL".into()))]
    pub disc_total: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Educe)]
#[educe(Default)]
pub struct VorbisComments {
    #[educe(Default(expression = Common::default_song()))]
    pub song: Common,
    #[educe(Default(expression = Common::default_album()))]
    pub album: Common,
    pub artists: Artists,
    pub track_disc: TrackDisc,
    #[educe(Default(expression = "LANGUAGE".into()))]
    pub languages: String,
    #[educe(Default(expression = "GENRE".into()))]
    pub genres: String,
    #[educe(Default(expression = "COMPILATION".into()))]
    pub compilation: String,
}

impl Common {
    fn default_song() -> Self {
        Self {
            name: "TITLE".into(),
            date: None,
            release_date: None,
            original_release_date: None,
            mbz_id: "MUSICBRAINZ_RELEASETRACKID".into(),
        }
    }

    fn default_album() -> Self {
        Self {
            name: "ALBUM".into(),
            date: Some("DATE".into()),
            release_date: Some("RELEASEDATE".into()),
            original_release_date: Some("ORIGYEAR".into()),
            mbz_id: "MUSICBRAINZ_ALBUMID".into(),
        }
    }
}

impl Artist {
    fn default_song() -> Self {
        Self { name: "ARTIST".into(), mbz_id: "MUSICBRAINZ_ARTISTID".into() }
    }

    fn default_album() -> Self {
        Self { name: "ALBUMARTIST".into(), mbz_id: "MUSICBRAINZ_ALBUMARTISTID".into() }
    }
}

#[cfg(test)]
#[coverage(off)]
mod test {
    use super::*;

    impl VorbisComments {
        pub fn test() -> Self {
            Self {
                song: Common {
                    date: Some("SDATE".into()),
                    release_date: Some("SRELEASEDATE".into()),
                    original_release_date: Some("SORIGYEAR".into()),
                    ..Common::default_song()
                },
                ..Self::default()
            }
        }
    }
}
