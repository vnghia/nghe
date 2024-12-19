use serde::Deserialize;
use serde_with::serde_as;
use uuid::Uuid;

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct Short {
    pub name: String,
    #[serde_as(as = "serde_with::NoneAsEmptyString")]
    #[serde(default)]
    pub mbid: Option<Uuid>,
    pub url: String,
}

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct Bio {
    pub summary: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Full {
    #[serde(flatten)]
    pub short: Short,
    pub bio: Bio,
}
