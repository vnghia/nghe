use serde::Deserialize;
use uuid::Uuid;

use crate::image::Image;

#[derive(Deserialize)]
pub struct Artist {
    pub name: String,
    pub mbid: Uuid,
    pub url: String,
    #[serde(rename = "image")]
    pub images: Vec<Image>,
}
