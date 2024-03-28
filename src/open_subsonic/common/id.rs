use std::str::FromStr;

use ::uuid::Uuid;
use anyhow::Result;
use concat_string::concat_string;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use crate::OSError;

const TYPED_ID_SEPARATOR: char = ':';
const TYPED_ID_STR: &str = TYPED_ID_SEPARATOR.as_ascii().unwrap().as_str();

#[derive(Debug, Clone, Copy)]
pub enum MediaType {
    Aritst,
    Album,
    Song,
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct TypedId<T> {
    pub t: Option<T>,
    pub id: Uuid,
}

pub type MediaTypedId = TypedId<MediaType>;

impl AsRef<str> for MediaType {
    fn as_ref(&self) -> &'static str {
        match self {
            MediaType::Aritst => "ar",
            MediaType::Album => "al",
            MediaType::Song => "so",
        }
    }
}

impl FromStr for MediaType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "ar" => Ok(MediaType::Aritst),
            "al" => Ok(MediaType::Album),
            "so" => Ok(MediaType::Song),
            _ => anyhow::bail!(OSError::InvalidParameter(
                concat_string::concat_string!("Value passed to enum DirectoryType {}", s).into()
            )),
        }
    }
}

impl<T: AsRef<str>> Serialize for TypedId<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut uuid_buffer = Uuid::encode_buffer();
        let uuid_str = self.id.hyphenated().encode_lower(&mut uuid_buffer);

        if let Some(ref t) = self.t {
            serializer.serialize_str(&concat_string!(t.as_ref(), TYPED_ID_STR, uuid_str))
        } else {
            serializer.serialize_str(uuid_str)
        }
    }
}

impl<'de, T: FromStr> Deserialize<'de> for TypedId<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if let Some((t, id)) = s.split_once(TYPED_ID_SEPARATOR) {
            Ok(Self {
                t: Some(
                    T::from_str(t)
                        .map_err(|_| de::Error::custom("could not construct type from string"))?,
                ),
                id: Uuid::from_str(id).map_err(de::Error::custom)?,
            })
        } else {
            Ok(Self { t: None, id: Uuid::from_str(&s).map_err(de::Error::custom)? })
        }
    }
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};
    use serde_json::{from_str, json, to_value};

    use super::*;

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct Test {
        pub id: TypedId<String>,
    }

    #[test]
    fn test_ser() {
        let id: Uuid = Faker.fake();
        let id_string = id.hyphenated().encode_lower(&mut Uuid::encode_buffer()).to_owned();

        let test = Test { id: TypedId { t: Some("type".to_owned()), id } };
        assert_eq!(
            to_value(test).unwrap(),
            json!({"id": concat_string!("type", TYPED_ID_STR, &id_string)})
        );

        let test = Test { id: TypedId { t: None, id } };
        assert_eq!(to_value(test).unwrap(), json!({"id": &id_string}));
    }

    #[test]
    fn test_der() {
        let id: Uuid = Faker.fake();
        let id_string = id.hyphenated().encode_lower(&mut Uuid::encode_buffer()).to_owned();

        let test = Test { id: TypedId { t: Some("type".to_owned()), id } };
        let data = json!({"id": concat_string!("type", TYPED_ID_STR, &id_string)}).to_string();
        assert_eq!(test, from_str(&data).unwrap());

        let test = Test { id: TypedId { t: None, id } };
        let data = json!({"id": &id_string}).to_string();
        assert_eq!(test, from_str(&data).unwrap());

        assert!(from_str::<Test>(&json!({"id": "invalid"}).to_string()).is_err());
        assert!(from_str::<Test>(&json!({"id": "invalid:uuid"}).to_string()).is_err());
    }
}
