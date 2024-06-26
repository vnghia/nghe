use diesel::prelude::*;
pub use songs_genres::*;
use uuid::Uuid;

pub use crate::schema::songs_genres;

#[derive(Insertable)]
#[diesel(table_name = songs_genres)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewSongGenre {
    pub song_id: Uuid,
    pub genre_id: Uuid,
}
