use derivative::Derivative;
use lofty::id3::v2::FrameId;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

fn serialize_frame_id<S>(f: &FrameId<'static>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(f.as_str())
}

fn deserialize_frame_id<'de, D>(deserializer: D) -> Result<FrameId<'static>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = <String>::deserialize(deserializer)?;
    FrameId::new(s)
        .map(|f| f.into_owned())
        .map_err(de::Error::custom)
}

#[derive(Debug, Serialize, Deserialize, Derivative)]
#[derivative(Default)]
pub struct Id3v2ParsingConfig {
    #[derivative(Default(value = "'/'"))]
    pub separator: char,
    #[serde(serialize_with = "serialize_frame_id")]
    #[serde(deserialize_with = "deserialize_frame_id")]
    #[derivative(Default(value = "FrameId::Valid(\"TPE1\".into())"))]
    pub artist: FrameId<'static>,
    #[serde(serialize_with = "serialize_frame_id")]
    #[serde(deserialize_with = "deserialize_frame_id")]
    #[derivative(Default(value = "FrameId::Valid(\"TPE2\".into())"))]
    pub album_artist: FrameId<'static>,
    #[serde(serialize_with = "serialize_frame_id")]
    #[serde(deserialize_with = "deserialize_frame_id")]
    #[derivative(Default(value = "FrameId::Valid(\"TRCK\".into())"))]
    pub track_number: FrameId<'static>,
    #[serde(serialize_with = "serialize_frame_id")]
    #[serde(deserialize_with = "deserialize_frame_id")]
    #[derivative(Default(value = "FrameId::Valid(\"TPOS\".into())"))]
    pub disc_number: FrameId<'static>,
    #[serde(serialize_with = "serialize_frame_id")]
    #[serde(deserialize_with = "deserialize_frame_id")]
    #[derivative(Default(value = "FrameId::Valid(\"TDRC\".into())"))]
    pub date: FrameId<'static>,
    #[serde(serialize_with = "serialize_frame_id")]
    #[serde(deserialize_with = "deserialize_frame_id")]
    #[derivative(Default(value = "FrameId::Valid(\"TDRL\".into())"))]
    pub release_date: FrameId<'static>,
    #[serde(serialize_with = "serialize_frame_id")]
    #[serde(deserialize_with = "deserialize_frame_id")]
    #[derivative(Default(value = "FrameId::Valid(\"TDOR\".into())"))]
    pub original_release_date: FrameId<'static>,
    #[serde(serialize_with = "serialize_frame_id")]
    #[serde(deserialize_with = "deserialize_frame_id")]
    #[derivative(Default(value = "FrameId::Valid(\"TLAN\".into())"))]
    pub language: FrameId<'static>,
}

#[derive(Debug, Serialize, Deserialize, Derivative)]
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
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ParsingConfig {
    pub id3v2: Id3v2ParsingConfig,
    pub vorbis: VorbisCommentsParsingConfig,
}
