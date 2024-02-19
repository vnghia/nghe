use super::artists::Artist;
use super::songs::Song;
pub use crate::schema::songs_artists;
pub use songs_artists::*;

use diesel::prelude::*;
use uuid::Uuid;

#[derive(
    Debug,
    Identifiable,
    Associations,
    Queryable,
    Selectable,
    Insertable,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
#[diesel(table_name = songs_artists)]
#[diesel(primary_key(song_id, artist_id))]
#[diesel(belongs_to(Song))]
#[diesel(belongs_to(Artist))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SongArtist {
    pub song_id: Uuid,
    pub artist_id: Uuid,
}

#[derive(Insertable)]
#[diesel(table_name = songs_artists)]
pub struct NewSongArtist<'a> {
    pub song_id: &'a Uuid,
    pub artist_id: &'a Uuid,
}
