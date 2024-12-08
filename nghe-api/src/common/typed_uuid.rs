use nghe_proc_macro::api_derive;
use uuid::Uuid;

#[api_derive(fake = true)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Artist,
    Album,
    Song,
}

#[api_derive(request = false, response = false, fake = true)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TypedUuid {
    pub ty: Type,
    pub id: Uuid,
}

mod serde {
    use core::str;
    use std::str::FromStr;

    use ::serde::{de, ser, Deserialize, Deserializer, Serialize, Serializer};
    use serde_with::DeserializeAs;

    use super::*;

    const SIMPLE_LENGTH: usize = uuid::fmt::Simple::LENGTH;
    const BUFFER_LENGTH: usize = SIMPLE_LENGTH + 2;
    const ARTIST_BYTE: &[u8; 2] = b"ar";
    const ALBUM_BYTE: &[u8; 2] = b"al";
    const SONG_BYTE: &[u8; 2] = b"so";

    impl Serialize for TypedUuid {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut buffer = [0u8; BUFFER_LENGTH];
            buffer[SIMPLE_LENGTH..BUFFER_LENGTH].copy_from_slice(match self.ty {
                Type::Artist => ARTIST_BYTE,
                Type::Album => ALBUM_BYTE,
                Type::Song => SONG_BYTE,
            });
            self.id.simple().encode_lower(&mut buffer);
            serializer.serialize_str(str::from_utf8(&buffer).map_err(ser::Error::custom)?)
        }
    }

    impl<'de> Deserialize<'de> for TypedUuid {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let buffer: [u8; BUFFER_LENGTH] = serde_with::Bytes::deserialize_as(deserializer)?;

            let ty_buffer: &[u8; 2] =
                &buffer[SIMPLE_LENGTH..BUFFER_LENGTH].try_into().map_err(de::Error::custom)?;
            let ty = match ty_buffer {
                ARTIST_BYTE => Type::Artist,
                ALBUM_BYTE => Type::Album,
                SONG_BYTE => Type::Song,
                _ => return Err(de::Error::custom("uuid type is invalid")),
            };

            let id = Uuid::from_str(
                str::from_utf8(&buffer[..SIMPLE_LENGTH]).map_err(de::Error::custom)?,
            )
            .map_err(de::Error::custom)?;

            Ok(Self { ty, id })
        }
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use fake::{Fake, Faker};
    use rstest::rstest;
    use uuid::uuid;

    use super::*;

    #[api_derive]
    #[derive(PartialEq)]
    struct Test {
        pub id: TypedUuid,
    }

    #[rstest]
    #[case(
        "id=a5b704dc8a7d4ff194896f8cc8890621ar",
        Some(TypedUuid { ty: Type::Artist, id: uuid!("a5b704dc-8a7d-4ff1-9489-6f8cc8890621") })
    )]
    #[case(
        "id=d22aee56cb844605b6a18e3b6d3aff72al",
        Some(TypedUuid { ty: Type::Album, id: uuid!("d22aee56-cb84-4605-b6a1-8e3b6d3aff72") })
    )]
    #[case(
        "id=e57894fcfcfb442fa09f9c5844232bd0so",
        Some(TypedUuid { ty: Type::Song, id: uuid!("e57894fc-fcfb-442f-a09f-9c5844232bd0") })
    )]
    #[case("id=e57894fcfcfb442fa09f9c5844232bd0in", None)]
    #[case("id=invalidar", None)]
    fn test_deserialize(#[case] input: &str, #[case] typed_uuid: Option<TypedUuid>) {
        assert_eq!(serde_html_form::from_str(input).ok(), typed_uuid.map(|id| Test { id }));
    }

    #[rstest]
    #[case(
        TypedUuid { ty: Type::Artist, id: uuid!("2fcc59a3-ae4e-496c-a981-74eb04827ef1") },
        "id=2fcc59a3ae4e496ca98174eb04827ef1ar",
    )]
    #[case(
        TypedUuid { ty: Type::Album, id: uuid!("3742eefb-1a9c-4c37-8653-ef07d12bdc31") },
        "id=3742eefb1a9c4c378653ef07d12bdc31al",
    )]
    #[case(
        TypedUuid { ty: Type::Song, id: uuid!("48c02916-0d4c-417d-ae61-1b1c36eb3e42") },
        "id=48c029160d4c417dae611b1c36eb3e42so",
    )]
    fn test_serialize(#[case] typed_uuid: TypedUuid, #[case] result: &str) {
        assert_eq!(serde_html_form::to_string(Test { id: typed_uuid }).unwrap(), result);
    }

    #[rstest]
    fn test_roundtrip() {
        let test = Test { id: Faker.fake() };
        assert_eq!(
            test,
            serde_html_form::from_str(&serde_html_form::to_string(&test).unwrap()).unwrap()
        );
    }
}
