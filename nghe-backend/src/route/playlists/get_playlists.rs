use diesel_async::RunQueryDsl;
use nghe_api::playlists::get_playlists::Playlists;
pub use nghe_api::playlists::get_playlists::{Request, Response};
use nghe_proc_macro::handler;
use uuid::Uuid;

use crate::database::Database;
use crate::orm::playlist;
use crate::Error;

#[handler]
pub async fn handler(database: &Database, user_id: Uuid) -> Result<Response, Error> {
    Ok(Response {
        playlists: Playlists {
            playlist: playlist::short::query::with_user_id(user_id)
                .get_results(&mut database.get().await?)
                .await?
                .into_iter()
                .map(playlist::short::Short::try_into)
                .try_collect()?,
        },
    })
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};
    use futures_lite::{stream, StreamExt as _};
    use rstest::rstest;

    use super::*;
    use crate::route::playlists::create_playlist;
    use crate::test::{mock, Mock};

    #[rstest]
    #[tokio::test]
    async fn test_handler(
        #[future(awt)]
        #[with(2, 1)]
        mock: Mock,
    ) {
        let mut music_folder = mock.music_folder(0).await;
        music_folder.add_audio().n_song((2..4).fake()).call().await;
        let song_ids: Vec<_> = music_folder.database.keys().copied().collect();

        let (user_ids, playlist_ids): (Vec<_>, Vec<Vec<_>>) = stream::iter(0..2)
            .then(async |i| {
                let user_id = mock.user_id(i).await;
                (
                    user_id,
                    stream::iter(0..(2..4).fake())
                        .then(async |_| {
                            create_playlist::handler(
                                mock.database(),
                                user_id,
                                create_playlist::Request {
                                    create_or_update: Faker.fake::<String>().into(),
                                    song_ids: Some(song_ids.clone()),
                                },
                            )
                            .await
                            .unwrap()
                            .playlist
                            .playlist
                            .id
                        })
                        .collect()
                        .await,
                )
            })
            .collect()
            .await;

        for i in 0..2 {
            let user_id = user_ids[i];
            let database_playlist_ids: Vec<_> = handler(mock.database(), user_id)
                .await
                .unwrap()
                .playlists
                .playlist
                .into_iter()
                .map(|playlist| playlist.id)
                .rev()
                .collect();
            assert_eq!(database_playlist_ids, playlist_ids[i]);
        }
    }
}
