use diesel::prelude::*;
pub use playlists_songs::*;
use uuid::Uuid;

pub use crate::schema::playlists_songs;

#[derive(Insertable)]
#[diesel(table_name = playlists_songs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AddSong {
    pub playlist_id: Uuid,
    pub song_id: Uuid,
}
