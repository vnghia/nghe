use std::borrow::Cow;

use educe::Educe;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{Error, database, error};

#[derive(Debug, Clone, Serialize, Deserialize, Educe)]
#[educe(Default)]
pub struct Index {
    #[serde(with = "split")]
    #[educe(Default(expression = Index::split("The An A Die Das Ein Eine Les Le La")))]
    pub ignore_prefixes: Vec<String>,
}

impl Index {
    fn split(s: &str) -> Vec<String> {
        s.split_ascii_whitespace().map(|v| concat_string::concat_string!(v, " ")).collect()
    }

    fn merge(prefixes: &[impl AsRef<str>]) -> Result<String, Error> {
        Ok(prefixes
            .iter()
            .map(|prefix| prefix.as_ref().strip_suffix(' '))
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| error::Kind::InvalidIndexIgnorePrefixesFormat)?
            .iter()
            .join(" "))
    }
}

impl database::Config for Index {
    const KEY: &'static str = "ignored_articles";

    const ENCRYPTED: bool = false;

    fn value(&self) -> Result<Cow<'_, str>, Error> {
        Self::merge(&self.ignore_prefixes).map(String::into)
    }
}

mod split {
    use serde::ser::Error;
    use serde::{Deserialize, Deserializer, Serializer};

    use super::*;

    pub fn serialize<S>(prefixes: &[String], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&Index::merge(prefixes).map_err(|_| {
            S::Error::custom(error::Kind::InvalidIndexIgnorePrefixesFormat.to_string())
        })?)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Index::split(&<String>::deserialize(deserializer)?))
    }
}
