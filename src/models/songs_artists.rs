use diesel::prelude::*;
pub use songs_artists::*;
use uuid::Uuid;

pub use crate::schema::songs_artists;

#[derive(Insertable)]
#[diesel(table_name = songs_artists)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewSongArtist {
    pub song_id: Uuid,
    pub artist_id: Uuid,
}
