use diesel::dsl::sql;
use diesel::expression::SqlLiteral;
use diesel::prelude::*;
use diesel::sql_types;
use uuid::Uuid;

use super::Album;
use crate::orm::id3::artist;

#[derive(Debug, Queryable, Selectable)]
pub struct WithArtistsSongs {
    #[diesel(embed)]
    pub album: Album,
    #[diesel(select_expression = sql(
        "array_agg(distinct(artists.id, artists.name) order by artists.name) album_artists"
    ))]
    #[diesel(select_expression_type =
        SqlLiteral::<sql_types::Array<sql_types::Record<(sql_types::Uuid, sql_types::Text)>>>
    )]
    pub artists: Vec<artist::Required>,
    #[diesel(select_expression = sql("bool_or(songs_album_artists.compilation) is_compilation"))]
    #[diesel(select_expression_type = SqlLiteral::<sql_types::Bool>)]
    pub is_compilation: bool,
    #[diesel(select_expression = sql("array_agg(distinct(songs.id)) album_artists"))]
    #[diesel(select_expression_type = SqlLiteral::<sql_types::Array<sql_types::Uuid>>)]
    pub songs: Vec<Uuid>,
}

pub mod query {
    use diesel::dsl::{auto_type, AsSelect};

    use super::*;
    use crate::orm::id3::album;
    use crate::orm::{albums, artists, songs, songs_album_artists};

    #[auto_type]
    pub fn unchecked() -> _ {
        let with_artists_songs: AsSelect<WithArtistsSongs, crate::orm::Type> =
            WithArtistsSongs::as_select();
        album::query::unchecked_no_group_by()
            .inner_join(songs_album_artists::table.on(songs_album_artists::song_id.eq(songs::id)))
            .inner_join(artists::table.on(artists::id.eq(songs_album_artists::album_artist_id)))
            .group_by(albums::id)
            .select(with_artists_songs)
    }
}
