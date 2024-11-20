use nghe_proc_macro::api_derive;
use uuid::Uuid;

use super::Song;
use crate::id3::genre;

#[serde_with::apply(
    genre::Genres => #[serde(skip_serializing_if = "genre::Genres::is_empty")],
)]
#[api_derive(response = true)]
pub struct WithAlbumGenres {
    #[serde(flatten)]
    pub song: Song,
    pub album: String,
    pub album_id: Uuid,
    pub genres: genre::Genres,
}
