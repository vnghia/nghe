use derivative::Derivative;
use lofty::id3::v2::FrameId;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde_with::{serde_as, DefaultOnNull};

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum FrameIdOrUserText {
    FrameId(FrameId<'static>),
    UserText(String),
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaDateMbzId3v2ParsingConfig {
    pub name: FrameIdOrUserText,
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub date: Option<FrameIdOrUserText>,
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub release_date: Option<FrameIdOrUserText>,
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub original_release_date: Option<FrameIdOrUserText>,
    pub mbz_id: FrameIdOrUserText,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Derivative)]
#[derivative(Default)]
pub struct Id3v2ParsingConfig {
    #[derivative(Default(value = "'/'"))]
    pub separator: char,

    #[derivative(Default(value = "MediaDateMbzId3v2ParsingConfig::default_song()"))]
    pub song: MediaDateMbzId3v2ParsingConfig,
    #[derivative(Default(value = "MediaDateMbzId3v2ParsingConfig::default_album()"))]
    pub album: MediaDateMbzId3v2ParsingConfig,

    #[derivative(Default(value = "\"TPE1\".try_into().unwrap()"))]
    pub artist: FrameIdOrUserText,
    #[derivative(Default(value = "\"TPE2\".try_into().unwrap()"))]
    pub album_artist: FrameIdOrUserText,

    #[derivative(Default(value = "\"TRCK\".try_into().unwrap()"))]
    pub track_number: FrameIdOrUserText,
    #[derivative(Default(value = "\"TPOS\".try_into().unwrap()"))]
    pub disc_number: FrameIdOrUserText,

    #[derivative(Default(value = "\"TLAN\".try_into().unwrap()"))]
    pub language: FrameIdOrUserText,

    #[derivative(Default(value = "\"MusicBrainz Artist Id\".try_into().unwrap()"))]
    pub artist_mbz_id: FrameIdOrUserText,
    #[derivative(Default(value = "\"MusicBrainz Album Artist Id\".try_into().unwrap()"))]
    pub album_artist_mbz_id: FrameIdOrUserText,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaDateMbzVorbisCommentsParsingConfig {
    pub name: String,
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub date: Option<String>,
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub release_date: Option<String>,
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub original_release_date: Option<String>,
    pub mbz_id: String,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Derivative)]
#[derivative(Default)]
pub struct VorbisCommentsParsingConfig {
    #[derivative(Default(value = "MediaDateMbzVorbisCommentsParsingConfig::default_song()"))]
    pub song: MediaDateMbzVorbisCommentsParsingConfig,
    #[derivative(Default(value = "MediaDateMbzVorbisCommentsParsingConfig::default_album()"))]
    pub album: MediaDateMbzVorbisCommentsParsingConfig,

    #[derivative(Default(value = "\"ARTIST\".into()"))]
    pub artist: String,
    #[derivative(Default(value = "\"ALBUMARTIST\".into()"))]
    pub album_artist: String,

    #[derivative(Default(value = "\"TRACKNUMBER\".into()"))]
    pub track_number: String,
    #[derivative(Default(value = "\"TRACKTOTAL\".into()"))]
    pub track_total: String,
    #[derivative(Default(value = "\"DISCNUMBER\".into()"))]
    pub disc_number: String,
    #[derivative(Default(value = "\"DISCTOTAL\".into()"))]
    pub disc_total: String,

    #[derivative(Default(value = "\"LANGUAGE\".into()"))]
    pub language: String,

    #[derivative(Default(value = "\"MUSICBRAINZ_ARTISTID\".into()"))]
    pub artist_mbz_id: String,
    #[derivative(Default(value = "\"MUSICBRAINZ_ALBUMARTISTID\".into()"))]
    pub album_artist_mbz_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ParsingConfig {
    pub id3v2: Id3v2ParsingConfig,
    pub vorbis: VorbisCommentsParsingConfig,
}

impl AsRef<str> for FrameIdOrUserText {
    fn as_ref(&self) -> &str {
        match self {
            FrameIdOrUserText::FrameId(frame_id) => frame_id.as_str(),
            FrameIdOrUserText::UserText(user_text) => user_text.as_str(),
        }
    }
}

impl TryFrom<String> for FrameIdOrUserText {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() == 4 || value.len() == 3 {
            FrameId::new(value).map(|f| Self::FrameId(f.into_owned())).map_err(Self::Error::from)
        } else {
            Ok(Self::UserText(value))
        }
    }
}

impl TryFrom<&str> for FrameIdOrUserText {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.to_string().try_into()
    }
}

impl Serialize for FrameIdOrUserText {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_ref())
    }
}

impl<'de> Deserialize<'de> for FrameIdOrUserText {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        <String>::deserialize(deserializer)?.try_into().map_err(de::Error::custom)
    }
}

impl MediaDateMbzId3v2ParsingConfig {
    fn default_song() -> Self {
        Self {
            name: "TIT2".try_into().unwrap(),
            date: Some("TRCS".try_into().unwrap()),
            release_date: Some("TSRL".try_into().unwrap()),
            original_release_date: Some("TSOR".try_into().unwrap()),
            mbz_id: "MusicBrainz Release Track Id".try_into().unwrap(),
        }
    }

    fn default_album() -> Self {
        Self {
            name: "TALB".try_into().unwrap(),
            date: Some("TDRC".try_into().unwrap()),
            release_date: Some("TDRL".try_into().unwrap()),
            original_release_date: Some("TDOR".try_into().unwrap()),
            mbz_id: "MusicBrainz Album Id".try_into().unwrap(),
        }
    }
}

impl MediaDateMbzVorbisCommentsParsingConfig {
    fn default_song() -> Self {
        Self {
            name: "TITLE".into(),
            date: Some("SDATE".into()),
            release_date: Some("SRELEASEDATE".into()),
            original_release_date: Some("SORIGYEAR".into()),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from() {
        assert_eq!(
            <FrameIdOrUserText as TryFrom<_>>::try_from("ABCD").unwrap(),
            FrameIdOrUserText::FrameId(FrameId::new("ABCD").unwrap().into_owned())
        );

        assert_eq!(
            <FrameIdOrUserText as TryFrom<_>>::try_from("ABCDEF").unwrap(),
            FrameIdOrUserText::UserText("ABCDEF".to_string())
        );
    }
}
