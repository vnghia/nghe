use derivative::Derivative;
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize, Derivative)]
#[derivative(Default)]
#[serde(default)]
pub struct Index {
    #[serde(with = "split")]
    #[derivative(Default(value = "Index::split(\"The An A Die Das Ein Eine Les Le La\")"))]
    pub ignore_prefixes: Vec<String>,
}

impl Index {
    fn split(s: &str) -> Vec<String> {
        s.split_ascii_whitespace().map(|v| concat_string::concat_string!(v, " ")).collect()
    }
}

mod split {
    use itertools::Itertools;
    use serde::ser::Error;
    use serde::{Deserialize, Deserializer, Serializer};

    use super::*;

    pub fn serialize<S>(prefixes: &[String], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(
            &prefixes
                .iter()
                .map(|prefix| prefix.strip_suffix(' '))
                .collect::<Option<Vec<_>>>()
                .ok_or_else(|| S::Error::custom("Prefix does not end with whitespace"))?
                .iter()
                .join(" "),
        )
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Index::split(&<String>::deserialize(deserializer)?))
    }
}
