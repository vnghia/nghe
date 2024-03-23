use diesel::prelude::*;
pub use songs_artists::*;
use uuid::Uuid;

pub use crate::schema::songs_artists;

#[derive(Insertable)]
#[diesel(table_name = songs_artists)]
pub struct NewSongArtist<'a> {
    pub song_id: &'a Uuid,
    pub artist_id: &'a Uuid,
}
