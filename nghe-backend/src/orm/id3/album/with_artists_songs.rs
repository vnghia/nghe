use diesel::dsl::sql;
use diesel::expression::SqlLiteral;
use diesel::prelude::*;
use diesel::sql_types;
use diesel_async::RunQueryDsl;
use nghe_api::id3;
use num_traits::ToPrimitive;
use uuid::Uuid;

use super::Album;
use crate::database::Database;
use crate::orm::id3::{artist, song};
use crate::orm::songs;
use crate::Error;

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

impl WithArtistsSongs {
    pub async fn try_into_api(
        self,
        database: &Database,
    ) -> Result<id3::album::WithArtistsSongs, Error> {
        let song: Vec<_> = songs::table
            .filter(songs::id.eq_any(self.songs))
            .order_by((songs::track_number.asc().nulls_last(), songs::title.asc()))
            .select(song::Song::as_select())
            .get_results(&mut database.get().await?)
            .await?;
        let duration: f32 = song.iter().map(|song| song.property.duration).sum();
        let song: Vec<_> = song.into_iter().map(song::Song::try_into_api).try_collect()?;

        let album = self
            .album
            .try_into_api_builder()?
            .song_count(song.len().try_into()?)
            .duration(
                duration
                    .ceil()
                    .to_u32()
                    .ok_or_else(|| Error::CouldNotConvertFloatToInteger(duration))?,
            )
            .build();

        let artists = self.artists.into_iter().map(artist::Required::into_api).collect();

        Ok(id3::album::WithArtistsSongs {
            album,
            artists,
            is_compilation: self.is_compilation,
            song,
        })
    }
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
