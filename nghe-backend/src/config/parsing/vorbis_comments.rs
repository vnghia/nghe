use derivative::Derivative;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Common {
    pub name: String,
    #[serde_as(deserialize_as = "serde_with::NoneAsEmptyString")]
    pub date: Option<String>,
    #[serde_as(deserialize_as = "serde_with::NoneAsEmptyString")]
    pub release_date: Option<String>,
    #[serde_as(deserialize_as = "serde_with::NoneAsEmptyString")]
    pub original_release_date: Option<String>,
    pub mbz_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artist {
    pub name: String,
    pub mbz_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Derivative)]
#[derivative(Default)]
pub struct Artists {
    #[derivative(Default(value = "Artist::default_song()"))]
    pub song: Artist,
    #[derivative(Default(value = "Artist::default_album()"))]
    pub album: Artist,
}

#[derive(Debug, Clone, Serialize, Deserialize, Derivative)]
#[derivative(Default)]
pub struct TrackDisc {
    #[derivative(Default(value = "\"TRACKNUMBER\".into()"))]
    pub track_number: String,
    #[derivative(Default(value = "\"TRACKTOTAL\".into()"))]
    pub track_total: String,
    #[derivative(Default(value = "\"DISCNUMBER\".into()"))]
    pub disc_number: String,
    #[derivative(Default(value = "\"DISCTOTAL\".into()"))]
    pub disc_total: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Derivative)]
#[derivative(Default)]
pub struct VorbisComments {
    #[derivative(Default(value = "Common::default_song()"))]
    pub song: Common,
    #[derivative(Default(value = "Common::default_album()"))]
    pub album: Common,
    pub artists: Artists,
    pub track_disc: TrackDisc,
    #[derivative(Default(value = "\"LANGUAGE\".into()"))]
    pub languages: String,
    #[derivative(Default(value = "\"GENRE\".into()"))]
    pub genres: String,
    #[derivative(Default(value = "\"COMPILATION\".into()"))]
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
