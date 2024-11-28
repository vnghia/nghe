use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
pub use nghe_api::browsing::get_song::{Request, Response};
use nghe_proc_macro::handler;
use uuid::Uuid;

use crate::database::Database;
use crate::orm::{id3, songs};
use crate::Error;

#[handler]
pub async fn handler(
    database: &Database,
    user_id: Uuid,
    request: Request,
) -> Result<Response, Error> {
    Ok(Response {
        song: id3::song::full::query::with_user_id(user_id)
            .filter(songs::id.eq(request.id))
            .get_result(&mut database.get().await?)
            .await?
            .try_into()?,
    })
}

#[cfg(test)]
mod test {
    use fake::{Fake, Faker};
    use itertools::Itertools;
    use rstest::rstest;

    use super::*;
    use crate::file::audio;
    use crate::test::{mock, Mock};

    #[rstest]
    #[tokio::test]
    async fn test_sorted(#[future(awt)] mock: Mock) {
        let mut music_folder = mock.music_folder(0).await;

        let album: audio::Album = Faker.fake();
        let album_id = album.upsert_mock(&mock, 0).await;
        let artists: Vec<_> = (0..(2..4).fake()).map(|i| i.to_string()).collect();
        music_folder
            .add_audio()
            .album(album.clone())
            .artists(audio::Artists {
                song: artists.clone().into_iter().map(String::into).collect(),
                album: [Faker.fake()].into(),
                compilation: false,
            })
            .genres(fake::vec![String; 0..4].iter().map(|genre| genre.to_lowercase()).collect())
            .call()
            .await;
        let song_id = music_folder.song_id(0);

        let database_song =
            handler(mock.database(), mock.user_id(0).await, Request { id: song_id })
                .await
                .unwrap()
                .song;

        assert_eq!(database_song.short.album, album.name);
        assert_eq!(database_song.short.album_id, album_id);

        let database_artists: Vec<_> =
            database_song.short.song.artists.into_iter().map(|artist| artist.name).collect();
        assert_eq!(database_artists, artists);

        let genres = database_song.genres.value;
        assert_eq!(genres, genres.iter().cloned().unique().sorted().collect::<Vec<_>>());
    }
}
