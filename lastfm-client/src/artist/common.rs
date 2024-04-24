use serde::Deserialize;
use serde_with::serde_as;
use uuid::Uuid;

#[serde_as]
#[derive(Deserialize)]
pub struct Artist {
    pub name: String,
    #[serde_as(as = "serde_with::NoneAsEmptyString")]
    #[serde(default)]
    pub mbid: Option<Uuid>,
    pub url: String,
}
