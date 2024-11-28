use nghe_proc_macro::api_derive;

use super::Short;
use crate::id3::genre;

#[api_derive]
pub struct Full {
    #[serde(flatten)]
    pub short: Short,
    pub genres: genre::Genres,
}
