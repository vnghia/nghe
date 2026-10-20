pub mod serde {
    use ::serde::{Deserialize, Deserializer, Serializer};
    use time::SignedDuration;

    pub fn serialize<S>(duration: &SignedDuration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let subsec = duration.subsec_milliseconds();
        serializer.serialize_i64(duration.whole_seconds() + i64::from(subsec > 0))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SignedDuration, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(SignedDuration::seconds(<i64>::deserialize(deserializer)?))
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
        duration: time::SignedDuration,
    }

    #[rstest]
    #[case(time::SignedDuration::seconds_f32(1.5), 2)]
    #[case(time::SignedDuration::seconds_f32(2.1), 3)]
    #[case(time::SignedDuration::seconds_f32(10.0), 10)]
    #[case(time::SignedDuration::seconds_f32(-9.5), -9)]
    #[case(time::SignedDuration::seconds_f32(-2.1), -2)]
    fn test_serialize_duration(#[case] duration: time::SignedDuration, #[case] result: i64) {
        assert_eq!(
            serde_json::to_string(&Test { duration }).unwrap(),
            serde_json::to_string(&json!({
                "duration": result,
            }))
            .unwrap()
        );
    }
}
