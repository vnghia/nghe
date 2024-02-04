use super::albums::Album;
use super::artists::Artist;
use super::songs::Song;
pub use crate::schema::albums_artists;
pub use albums_artists::*;

use diesel::prelude::*;
use time::OffsetDateTime;
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
#[diesel(table_name = albums_artists)]
#[diesel(primary_key(album_id, artist_id, song_id))]
#[diesel(belongs_to(Album))]
#[diesel(belongs_to(Artist))]
#[diesel(belongs_to(Song))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AlbumArtist {
    pub album_id: Uuid,
    pub artist_id: Uuid,
    pub song_id: Uuid,
    pub upserted_at: OffsetDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = albums_artists)]
pub struct NewAlbumArtist {
    pub album_id: Uuid,
    pub artist_id: Uuid,
    pub song_id: Uuid,
}
