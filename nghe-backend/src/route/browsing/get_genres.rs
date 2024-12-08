use diesel_async::RunQueryDsl;
use nghe_api::browsing::get_genres::Genres;
pub use nghe_api::browsing::get_genres::{Request, Response};
use nghe_proc_macro::handler;
use uuid::Uuid;

use crate::database::Database;
use crate::orm::id3;
use crate::Error;

#[handler]
pub async fn handler(database: &Database, user_id: Uuid) -> Result<Response, Error> {
    Ok(Response {
        genres: Genres {
            genre: id3::genre::with_count::query::with_user_id(user_id)
                .get_results(&mut database.get().await?)
                .await?
                .into_iter()
                .map(id3::genre::with_count::WithCount::try_into)
                .try_collect()?,
        },
    })
}

#[cfg(test)]
#[coverage(off)]
mod test {
    use fake::{Fake, Faker};
    use rstest::rstest;

    use super::*;
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
        mock.add_music_folder().call().await;

        let mut music_folder_permission = mock.music_folder(0).await;
        let mut music_folder = mock.music_folder(1).await;

        let genre: String = Faker.fake();

        let n_song_permission = (2..4).fake();
        let n_song = (2..4).fake();

        music_folder_permission
            .add_audio()
            .genres([genre.clone(), Faker.fake()].into_iter().collect())
            .n_song(n_song_permission)
            .call()
            .await;
        music_folder
            .add_audio()
            .genres([genre.clone(), Faker.fake()].into_iter().collect())
            .n_song(n_song)
            .call()
            .await;

        let genres = handler(mock.database(), mock.user_id(0).await).await.unwrap().genres.genre;
        assert_eq!(genres.len(), if allow { 3 } else { 2 });

        let genre = genres.into_iter().find(|with_count| with_count.value == genre).unwrap();
        let count: u32 =
            (if allow { n_song_permission + n_song } else { n_song }).try_into().unwrap();
        assert_eq!(genre.song_count, count);
        assert_eq!(genre.album_count, count);
    }
}
