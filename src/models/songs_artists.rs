pub use crate::schema::songs_artists;
pub use songs_artists::*;

use diesel::prelude::*;
use uuid::Uuid;

#[derive(Insertable)]
#[diesel(table_name = songs_artists)]
pub struct NewSongArtist<'a> {
    pub song_id: &'a Uuid,
    pub artist_id: &'a Uuid,
}
