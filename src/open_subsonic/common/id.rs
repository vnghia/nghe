#[cfg(test)]
mod tests {
    use concat_string::concat_string;
    use fake::{Fake, Faker};
    use nghe_types::id::{MediaType, MediaTypedId, TypedId, TYPED_ID_STR};
    use serde::{Deserialize, Serialize};
    use serde_json::{from_str, json, to_value};
    use uuid::Uuid;

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct Test {
        pub id: MediaTypedId,
    }

    #[test]
    fn test_ser() {
        let id: Uuid = Faker.fake();
        let id_string = id.hyphenated().encode_lower(&mut Uuid::encode_buffer()).to_owned();

        let test = Test { id: MediaTypedId { t: Some(MediaType::Aritst), id } };
        assert_eq!(
            to_value(test).unwrap(),
            json!({"id": concat_string!("ar", TYPED_ID_STR, &id_string)})
        );

        let test = Test { id: TypedId { t: None, id } };
        assert_eq!(to_value(test).unwrap(), json!({"id": &id_string}));
    }

    #[test]
    fn test_der() {
        let id: Uuid = Faker.fake();
        let id_string = id.hyphenated().encode_lower(&mut Uuid::encode_buffer()).to_owned();

        let test = Test { id: MediaTypedId { t: Some(MediaType::Album), id } };
        let data = json!({"id": concat_string!("al", TYPED_ID_STR, &id_string)}).to_string();
        assert_eq!(test, from_str(&data).unwrap());

        let test = Test { id: TypedId { t: None, id } };
        let data = json!({"id": &id_string}).to_string();
        assert_eq!(test, from_str(&data).unwrap());

        assert!(from_str::<Test>(&json!({"id": "invalid"}).to_string()).is_err());
        assert!(from_str::<Test>(&json!({"id": "invalid:uuid"}).to_string()).is_err());
    }
}
