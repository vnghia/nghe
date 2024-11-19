use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
pub use nghe_api::browsing::get_artist::{Request, Response};
use nghe_proc_macro::handler;
use uuid::Uuid;

use crate::database::Database;
use crate::error::Error;
use crate::orm::{artists, id3};

#[handler]
pub async fn handler(
    database: &Database,
    user_id: Uuid,
    request: Request,
) -> Result<Response, Error> {
    Ok(Response {
        artist: id3::artist_with_albums::query::with_user_id(user_id)
            .filter(artists::id.eq(request.id))
            .get_result(&mut database.get().await?)
            .await?
            .try_into_api(database)
            .await?,
    })
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};
    use rstest::rstest;

    use super::*;
    use crate::file::audio;
    use crate::test::{mock, Mock};

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
                .call()
                .await;
        }

        let artist = handler(mock.database(), mock.user_id(0).await, Request { id: artist_id })
            .await
            .unwrap()
            .artist;
        assert_eq!(
            artist.album.into_iter().map(|album| album.name).collect::<Vec<_>>(),
            (0..n_album).map(|i| i.to_string()).collect::<Vec<_>>()
        );
    }
}
