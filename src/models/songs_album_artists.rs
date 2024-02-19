use super::artists::Artist;
use super::songs::Song;
pub use crate::schema::songs_album_artists;
pub use songs_album_artists::*;

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
#[diesel(table_name = songs_album_artists)]
#[diesel(primary_key(song_id, album_artist_id))]
#[diesel(belongs_to(Song))]
#[diesel(belongs_to(Artist, foreign_key = album_artist_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SongAlbumArtist {
    pub song_id: Uuid,
    pub album_artist_id: Uuid,
}

#[derive(Insertable)]
#[diesel(table_name = songs_album_artists)]
pub struct NewSongAlbumArtist<'a> {
    pub song_id: &'a Uuid,
    pub album_artist_id: &'a Uuid,
}
