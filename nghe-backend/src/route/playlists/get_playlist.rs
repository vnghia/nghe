use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
pub use nghe_api::playlists::get_playlist::{Request, Response};
use nghe_proc_macro::handler;
use uuid::Uuid;

use crate::database::Database;
use crate::orm::{playlist, playlists};
use crate::Error;

#[handler]
pub async fn handler(
    database: &Database,
    user_id: Uuid,
    request: Request,
) -> Result<Response, Error> {
    let playlist_id = request.id;
    Ok(Response {
        playlist: playlist::full::query::with_user_id(user_id)
            .filter(playlists::id.eq(playlist_id))
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
    use futures_lite::{stream, StreamExt as _};
    use rstest::rstest;

    use super::*;
    use crate::route::playlists::create_playlist;
    use crate::test::{mock, Mock};

    #[rstest]
    #[tokio::test]
    async fn test_handler_music_folder(
        #[future(awt)]
        #[with(1, 0)]
        mock: Mock,
        #[values(true, false)] allow: bool,
    ) {
        mock.add_music_folder().allow(allow).call().await;
        mock.add_music_folder().call().await;

        let mut music_folder_permission = mock.music_folder(0).await;
        let mut music_folder = mock.music_folder(1).await;

        let n_song_permission = (2..4).fake();
        let n_song = (2..4).fake();
        music_folder_permission.add_audio().n_song(n_song_permission).call().await;
        music_folder.add_audio().n_song(n_song).call().await;

        let song_ids: Vec<_> = music_folder_permission
            .database
            .keys()
            .copied()
            .chain(music_folder.database.keys().copied())
            .collect();

        let playlist = create_playlist::handler(
            mock.database(),
            mock.user_id(0).await,
            create_playlist::Request {
                create_or_update: Faker.fake::<String>().into(),
                song_ids: Some(song_ids.clone()),
            },
        )
        .await
        .unwrap()
        .playlist;

        let database_song_ids: Vec<_> = playlist.entry.iter().map(|entry| entry.song.id).collect();
        let index = if allow { 0 } else { n_song_permission };
        assert_eq!(database_song_ids, song_ids[index..]);
    }

    #[rstest]
    #[tokio::test]
    async fn test_handler_user(
        #[future(awt)]
        #[with(2, 1)]
        mock: Mock,
    ) {
        let mut music_folder = mock.music_folder(0).await;
        music_folder.add_audio().n_song((2..4).fake()).call().await;
        let song_ids: Vec<_> = music_folder.database.keys().copied().collect();

        let (user_ids, playlist_ids): (Vec<_>, Vec<_>) = stream::iter(0..2)
            .then(async |i| {
                let user_id = mock.user_id(i).await;
                (
                    user_id,
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
                    .id,
                )
            })
            .collect()
            .await;

        for i in 0..2 {
            let user_id = user_ids[i];
            let playlist_id = playlist_ids[1 - i];
            assert!(handler(mock.database(), user_id, Request { id: playlist_id }).await.is_err());
        }
    }
}
