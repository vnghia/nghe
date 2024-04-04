use derivative::Derivative;
use lofty::id3::v2::FrameId;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum FrameIdOrUserText {
    FrameId(FrameId<'static>),
    UserText(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Derivative)]
#[derivative(Default)]
pub struct Id3v2ParsingConfig {
    #[derivative(Default(value = "'/'"))]
    pub separator: char,
    #[derivative(Default(value = "\"TPE1\".try_into().unwrap()"))]
    pub artist: FrameIdOrUserText,
    #[derivative(Default(value = "\"TPE2\".try_into().unwrap()"))]
    pub album_artist: FrameIdOrUserText,
    #[derivative(Default(value = "\"TRCK\".try_into().unwrap()"))]
    pub track_number: FrameIdOrUserText,
    #[derivative(Default(value = "\"TPOS\".try_into().unwrap()"))]
    pub disc_number: FrameIdOrUserText,
    #[derivative(Default(value = "\"TDRC\".try_into().unwrap()"))]
    pub date: FrameIdOrUserText,
    #[derivative(Default(value = "\"TDRL\".try_into().unwrap()"))]
    pub release_date: FrameIdOrUserText,
    #[derivative(Default(value = "\"TDOR\".try_into().unwrap()"))]
    pub original_release_date: FrameIdOrUserText,
    #[derivative(Default(value = "\"TLAN\".try_into().unwrap()"))]
    pub language: FrameIdOrUserText,
    #[derivative(Default(value = "\"MusicBrainz Artist Id\".try_into().unwrap()"))]
    pub artist_mbz_id: FrameIdOrUserText,
    #[derivative(Default(value = "\"MusicBrainz Album Artist Id\".try_into().unwrap()"))]
    pub album_artist_mbz_id: FrameIdOrUserText,
}

#[derive(Debug, Clone, Serialize, Deserialize, Derivative)]
#[derivative(Default)]
pub struct VorbisCommentsParsingConfig {
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
    #[derivative(Default(value = "\"DATE\".into()"))]
    pub date: String,
    #[derivative(Default(value = "\"RELEASEDATE\".into()"))]
    pub release_date: String,
    #[derivative(Default(value = "\"ORIGYEAR\".into()"))]
    pub original_release_date: String,
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
