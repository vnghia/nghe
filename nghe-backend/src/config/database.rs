use derivative::Derivative;
use serde::Deserialize;
use serde_with::serde_as;

use crate::app::Key;

#[serde_as]
#[derive(Derivative, Deserialize)]
#[derivative(Debug)]
pub struct Database {
    #[derivative(Debug = "ignore")]
    pub url: String,
    #[derivative(Debug = "ignore")]
    #[serde_as(as = "serde_with::hex::Hex")]
    pub key: Key,
}
