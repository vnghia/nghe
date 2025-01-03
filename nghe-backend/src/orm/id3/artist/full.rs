use diesel::dsl::sql;
use diesel::expression::SqlLiteral;
use diesel::prelude::*;
use diesel::sql_types;
use diesel_async::RunQueryDsl;
use nghe_api::id3;
use uuid::Uuid;

use super::super::album;
use super::Artist;
use crate::Error;
use crate::database::Database;
use crate::orm::albums;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = artists, check_for_backend(crate::orm::Type))]
pub struct Full {
    #[diesel(embed)]
    pub artist: Artist,
    #[diesel(select_expression = sql(
        "array_remove(array_agg(distinct(albums.id)), null) album_ids"
    ))]
    #[diesel(select_expression_type = SqlLiteral::<sql_types::Array<sql_types::Uuid>>)]
    pub albums: Vec<Uuid>,
}

impl Full {
    pub async fn try_into(
        self,
        database: &Database,
        user_id: Uuid,
    ) -> Result<id3::artist::Full, Error> {
        Ok(id3::artist::Full {
            artist: self.artist.try_into()?,
            album: album::short::query::with_user_id_unchecked(user_id)
                .filter(albums::id.eq_any(self.albums))
                .get_results(&mut database.get().await?)
                .await?
                .into_iter()
                .map(album::short::Short::try_into)
                .try_collect()?,
        })
    }
}

pub mod query {
    use diesel::dsl::{AsSelect, auto_type};
    use uuid::Uuid;

    use super::*;
    use crate::orm::id3::artist;

    #[auto_type]
    pub fn with_user_id(user_id: Uuid) -> _ {
        let artist: artist::query::with_user_id = artist::query::with_user_id(user_id);
        let full: AsSelect<Full, crate::orm::Type> = Full::as_select();
        artist.select(full)
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use diesel_async::RunQueryDsl;
    use fake::{Fake, Faker};
    use rstest::rstest;

    use super::*;
    use crate::file::audio;
    use crate::schema::artists;
    use crate::test::{Mock, mock};

    #[rstest]
    #[tokio::test]
    async fn test_query_artist(#[future(awt)] mock: Mock, #[values(0, 6)] n_album: i64) {
        let artist: audio::Artist = Faker.fake();
        let artist_id = artist.upsert_mock(&mock).await;

        mock.add_audio_artist(0, [artist.clone()], [Faker.fake()], false, 1).await;
        mock.add_audio_artist(
            0,
            [Faker.fake()],
            [artist.clone()],
            false,
            n_album.try_into().unwrap(),
        )
        .await;

        let database_artist = query::with_user_id(mock.user_id(0).await)
            .filter(artists::id.eq(artist_id))
            .get_result(&mut mock.get().await)
            .await
            .unwrap();

        assert_eq!(database_artist.artist.album_count, n_album);
        let n_album: usize = n_album.try_into().unwrap();
        assert_eq!(database_artist.albums.len(), n_album);
    }

    #[rstest]
    #[tokio::test]
    async fn test_query_partial(
        #[future(awt)]
        #[with(1, 0)]
        mock: Mock,
        #[values(true, false)] allow: bool,
    ) {
        mock.add_music_folder().allow(allow).call().await;
        mock.add_music_folder().call().await;

        let artist: audio::Artist = Faker.fake();
        let artist_id = artist.upsert_mock(&mock).await;

        let n_album = (2..4).fake();

        let album: audio::Album = Faker.fake();
        let album_id = album.upsert_mock(&mock, 0).await;
        mock.music_folder(0)
            .await
            .add_audio()
            .album(album)
            .artists(audio::Artists {
                album: [artist.clone()].into(),
                compilation: false,
                ..Faker.fake()
            })
            .call()
            .await;

        mock.add_audio_artist(1, [Faker.fake()], [artist.clone()], false, n_album).await;

        let database_artist = query::with_user_id(mock.user_id(0).await)
            .filter(artists::id.eq(artist_id))
            .get_result(&mut mock.get().await)
            .await
            .unwrap();

        let n_album = if allow { n_album + 1 } else { n_album };
        assert_eq!(database_artist.albums.len(), n_album);
        let n_album: i64 = n_album.try_into().unwrap();
        assert_eq!(database_artist.artist.album_count, n_album);
        assert_eq!(database_artist.albums.contains(&album_id), allow);
    }
}
