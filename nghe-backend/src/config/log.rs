use educe::Educe;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Format {
    #[default]
    Plain,
    Json,
}

#[derive(Debug, Serialize, Deserialize, Educe)]
#[educe(Default)]
pub struct Log {
    #[educe(Default(expression = true))]
    pub time: bool,
    pub format: Format,
}
