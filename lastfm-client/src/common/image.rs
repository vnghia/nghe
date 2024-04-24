use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "size", content = "#text")]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum Image {
    Small(String),
    Medium(String),
    Large(String),
    ExtraLarge(String),
    Mega(String),
    #[serde(rename = "")]
    Empty(String),
}
