use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
pub use nghe_api::browsing::get_artist::{Request, Response};
use nghe_proc_macro::handler;
use uuid::Uuid;

use crate::Error;
use crate::database::Database;
use crate::orm::{artists, id3};

#[handler]
pub async fn handler(
    database: &Database,
    user_id: Uuid,
    request: Request,
) -> Result<Response, Error> {
    Ok(Response {
        artist: id3::artist::full::query::with_user_id(user_id)
            .filter(artists::id.eq(request.id))
            .get_result(&mut database.get().await?)
            .await?
            .try_into(database)
            .await?,
    })
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use fake::{Fake, Faker};
    use itertools::Itertools;
    use rstest::rstest;

    use super::*;
    use crate::file::audio;
    use crate::test::{Mock, mock};

    #[rstest]
    #[tokio::test]
    async fn test_sorted(#[future(awt)] mock: Mock) {
        let mut music_folder = mock.music_folder(0).await;

        let artist: audio::Artist = Faker.fake();
        let artist_id = artist.upsert_mock(&mock).await;

        let n_album = (2..4).fake();
        for i in 0..n_album {
            music_folder
                .add_audio()
                .album(i.to_string().into())
                .artists(audio::Artists {
                    album: [artist.clone()].into(),
                    compilation: false,
                    ..Faker.fake()
                })
                .genres(fake::vec![String; 0..4].iter().map(|genre| genre.to_lowercase()).collect())
                .call()
                .await;
        }

        let artist = handler(mock.database(), mock.user_id(0).await, Request { id: artist_id })
            .await
            .unwrap()
            .artist;

        let n_album: usize = n_album.try_into().unwrap();
        assert_eq!(artist.album.len(), n_album);
        for (i, album) in artist.album.into_iter().enumerate() {
            assert_eq!(album.name, i.to_string());
            let genres = album.genres.value;
            assert_eq!(genres, genres.iter().cloned().unique().sorted().collect::<Vec<_>>());
        }
    }
}
