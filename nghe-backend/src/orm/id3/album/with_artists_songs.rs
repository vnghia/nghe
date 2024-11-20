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
    #[diesel(embed)]
    pub artists: artist::required::Artists,
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

        Ok(id3::album::WithArtistsSongs {
            album,
            artists: self.artists.into(),
            is_compilation: self.is_compilation,
            song,
        })
    }
}

pub mod query {
    use diesel::dsl::{auto_type, AsSelect};

    use super::*;
    use crate::orm::id3::album;
    use crate::orm::{albums, permission, songs, songs_album_artists};

    #[auto_type]
    pub fn unchecked() -> _ {
        let with_artists_songs: AsSelect<WithArtistsSongs, crate::orm::Type> =
            WithArtistsSongs::as_select();
        album::query::unchecked_no_group_by()
            .inner_join(songs_album_artists::table.on(songs_album_artists::song_id.eq(songs::id)))
            .inner_join(artist::required::query::album())
            .group_by(albums::id)
            .select(with_artists_songs)
    }

    #[auto_type]
    pub fn with_user_id(user_id: Uuid) -> _ {
        let permission: permission::with_album = permission::with_album(user_id);
        unchecked().filter(permission)
    }
}

#[cfg(test)]
mod tests {
    use diesel_async::RunQueryDsl;
    use fake::{Fake, Faker};
    use indexmap::IndexSet;
    use rstest::rstest;

    use super::*;
    use crate::file::audio;
    use crate::orm::albums;
    use crate::test::{mock, Mock};

    #[rstest]
    #[tokio::test]
    async fn test_query(
        #[future(awt)]
        #[with(1, 0)]
        mock: Mock,
        #[values(true, false)] allow: bool,
    ) {
        mock.add_music_folder().allow(allow).call().await;
        let mut music_folder = mock.music_folder(0).await;

        let album: audio::Album = Faker.fake();
        let album_id = album.upsert_mock(&mock, 0).await;

        let n_song = (2..4).fake();
        for i in 0..n_song {
            music_folder
                .add_audio()
                .album(album.clone())
                .song(audio::Song {
                    track_disc: audio::TrackDisc {
                        track: audio::position::Position {
                            number: Some((i + 1).try_into().unwrap()),
                            ..Faker.fake()
                        },
                        ..Faker.fake()
                    },
                    ..Faker.fake()
                })
                .call()
                .await;
        }

        let database_album = query::with_user_id(mock.user_id(0).await)
            .filter(albums::id.eq(album_id))
            .get_result(&mut mock.get().await)
            .await;

        if allow {
            let database_album = database_album.unwrap();
            assert_eq!(
                database_album.songs.iter().collect::<IndexSet<_>>(),
                music_folder.database.keys().collect::<IndexSet<_>>()
            );
        } else {
            assert!(database_album.is_err());
        }
    }
}
