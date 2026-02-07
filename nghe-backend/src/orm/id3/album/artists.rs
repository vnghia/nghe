use diesel_async::RunQueryDsl;
use nghe_api::id3;
use uuid::Uuid;

use crate::Error;
use crate::database::Database;
use crate::orm::id3::artist;
use crate::orm::{artists, songs_album_artists};

pub struct Artists;

impl Artists {
    pub async fn query(
        database: &Database,
        album_id: Uuid,
    ) -> Result<Vec<id3::artist::Required>, Error> {
        Ok(query::with_album_id_unchecked(album_id)
            .get_results(&mut database.get().await?)
            .await?
            .into_iter()
            .map(artist::required::Required::into)
            .collect())
    }
}

mod query {
    use diesel::dsl::{AsSelect, auto_type, min, not};
    use diesel::{ExpressionMethods, PgSortExpressionMethods, QueryDsl, SelectableHelper};

    use super::*;
    use crate::orm::{albums, songs};

    #[auto_type]
    pub fn with_album_id_unchecked(album_id: Uuid) -> _ {
        let required: AsSelect<artist::required::Required, crate::orm::Type> =
            artist::required::Required::as_select();
        // Grouped by artists::id so we will not have any duplication.
        songs::table
            .inner_join(songs_album_artists::table)
            .inner_join(albums::table)
            .inner_join(artist::required::query::album())
            .filter(albums::id.eq(album_id))
            .filter(not(songs_album_artists::compilation))
            .group_by(artists::id)
            .order_by((
                min(songs::disc_number).asc().nulls_first(),
                min(songs::track_number).asc().nulls_first(),
                min(songs_album_artists::upserted_at).asc(),
            ))
            .select(required)
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use fake::{Fake, Faker};
    use rstest::rstest;

    use super::*;
    use crate::file::audio;
    use crate::test::{Mock, mock};

    #[rstest]
    #[tokio::test]
    async fn test_query(#[future(awt)] mock: Mock, #[values(true, false)] compilation: bool) {
        let mut music_folder = mock.music_folder(0).await;

        let artist_names_1 = fake::vec![String; 2];
        let artist_names_2: Vec<_> =
            fake::vec![String; 1].into_iter().chain(artist_names_1[0..1].iter().cloned()).collect();
        let artist_names: Vec<_> =
            artist_names_1[0..2].iter().chain(artist_names_2[0..1].iter()).cloned().collect();

        let album: audio::Album = Faker.fake();
        let album_id = album.upsert_mock(&mock, 0).await;

        music_folder
            .add_audio()
            .album(album.clone())
            .artists(
                audio::Artists::new(
                    fake::vec![audio::Artist; 2..4],
                    artist_names_1.clone().into_iter().map(std::convert::Into::into),
                    compilation,
                )
                .unwrap(),
            )
            .song(audio::Song {
                track_disc: audio::TrackDisc {
                    track: audio::position::Position { number: Some(1), total: None },
                    disc: audio::position::Position::default(),
                },
                ..Faker.fake()
            })
            .call()
            .await;

        music_folder
            .add_audio()
            .album(album.clone())
            .artists(
                audio::Artists::new(
                    fake::vec![audio::Artist; 2..4],
                    artist_names_2.clone().into_iter().map(std::convert::Into::into),
                    compilation,
                )
                .unwrap(),
            )
            .song(audio::Song {
                track_disc: audio::TrackDisc {
                    track: audio::position::Position { number: Some(2), total: None },
                    disc: audio::position::Position::default(),
                },
                ..Faker.fake()
            })
            .call()
            .await;

        let artists: Vec<String> = Artists::query(mock.database(), album_id)
            .await
            .unwrap()
            .into_iter()
            .map(|artist| artist.name)
            .collect();
        assert_eq!(artists, artist_names);
    }
}
