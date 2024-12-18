use std::str::FromStr;

use lofty::id3::v2::FrameId;
use strum::{EnumDiscriminants, EnumString, IntoStaticStr};

use crate::{Error, error};

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumString, IntoStaticStr))]
#[cfg_attr(test, derive(PartialEq))]
pub enum Id {
    #[strum_discriminants(strum(serialize = "TEXT"))]
    Text(FrameId<'static>),
    #[strum_discriminants(strum(serialize = "TXXX"))]
    UserText(String),
    #[strum_discriminants(strum(serialize = "TIME"))]
    Time(FrameId<'static>),
}

impl Id {
    fn as_str(&self) -> &str {
        match self {
            Id::UserText(description) => description,
            Id::Text(frame_id) | Id::Time(frame_id) => frame_id.as_str(),
        }
    }
}

impl FromStr for Id {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (variant, id) =
            s.split_once(':').ok_or_else(|| error::Kind::InvalidId3v2FrameIdStringFormat)?;
        let variant: IdDiscriminants =
            variant.parse().map_err(|_| error::Kind::InvalidId3v2FrameIdStringFormat)?;
        let id = id.to_owned();
        Ok(match variant {
            IdDiscriminants::Text => Self::Text(
                FrameId::new(id).map_err(|_| error::Kind::InvalidId3v2FrameIdStringFormat)?,
            ),
            IdDiscriminants::UserText => Self::UserText(id),
            IdDiscriminants::Time => Self::Time(
                FrameId::new(id).map_err(|_| error::Kind::InvalidId3v2FrameIdStringFormat)?,
            ),
        })
    }
}

mod serde {
    use ::serde::{Deserialize, Deserializer, Serialize, Serializer, de};
    use concat_string::concat_string;

    use super::*;

    impl Serialize for Id {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let variant: IdDiscriminants = self.into();
            let variant: &'static str = variant.into();
            serializer.serialize_str(&concat_string!(variant, ":", self.as_str()))
        }
    }

    impl<'de> Deserialize<'de> for Id {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            <String>::deserialize(deserializer)?
                .parse()
                .map_err(|error: Error| de::Error::custom(error.source))
        }
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use nghe_proc_macro::api_derive;
    use rstest::rstest;

    use super::*;

    #[api_derive]
    #[derive(PartialEq)]
    struct Test {
        pub id: Id,
    }

    #[rstest]
    #[case("TEXT:IDID", Some(Id::Text(FrameId::Valid("IDID".to_owned().into()))))]
    #[case("TXXX:Test description", Some(Id::UserText("Test description".to_owned())))]
    #[case("TIME:IDID", Some(Id::Time(FrameId::Valid("IDID".to_owned().into()))))]
    #[case("Invalid", None)]
    fn test_deserialize(#[case] input: &str, #[case] id: Option<Id>) {
        assert_eq!(
            serde_json::from_value(serde_json::json!({"id": input})).ok(),
            id.map(|id| Test { id })
        );
    }

    #[rstest]
    #[case(Id::Text(FrameId::Valid("IDID".to_owned().into())), "TEXT:IDID")]
    #[case(Id::UserText("Test description".to_owned()), "TXXX:Test description")]
    #[case(Id::Time(FrameId::Valid("IDID".to_owned().into())), "TIME:IDID")]
    fn test_serialize(#[case] id: Id, #[case] result: &str) {
        assert_eq!(
            serde_json::to_string(&Test { id }).unwrap(),
            serde_json::to_string(&serde_json::json!({"id": result})).unwrap()
        );
    }
}
