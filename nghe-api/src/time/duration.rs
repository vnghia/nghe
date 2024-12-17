pub mod serde {
    use ::serde::{Deserialize, Deserializer, Serializer, de, ser};
    use num_traits::ToPrimitive;
    use time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(
            duration
                .as_seconds_f32()
                .ceil()
                .to_u32()
                .ok_or_else(|| ser::Error::custom("Could not serialize duration to integer"))?,
        )
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Duration::seconds_f32(
            <u32>::deserialize(deserializer)?
                .to_f32()
                .ok_or_else(|| de::Error::custom("Could not deserialize duration from integer"))?,
        ))
    }
}
