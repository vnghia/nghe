use diesel::prelude::*;
pub use songs_album_artists::*;
use uuid::Uuid;

pub use crate::schema::songs_album_artists;

#[derive(Insertable)]
#[diesel(table_name = songs_album_artists)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewSongAlbumArtist {
    pub song_id: Uuid,
    pub album_artist_id: Uuid,
    pub compilation: bool,
}
