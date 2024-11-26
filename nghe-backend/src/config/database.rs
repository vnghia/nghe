use educe::Educe;
use serde::Deserialize;
use serde_with::serde_as;

use crate::database::Key;

#[serde_as]
#[derive(Deserialize, Educe)]
#[educe(Debug)]
pub struct Database {
    #[educe(Debug(ignore))]
    pub url: String,
    #[serde_as(as = "serde_with::hex::Hex")]
    #[educe(Debug(ignore))]
    pub key: Key,
}
