use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl};
use diesel_async::RunQueryDsl;
pub use nghe_api::playlists::update_playlist::{Request, Response};
use nghe_proc_macro::handler;
use uuid::Uuid;

use crate::database::Database;
use crate::orm::upsert::Update;
use crate::orm::{albums, playlist, playlists, playlists_songs, songs};
use crate::Error;

#[handler]
pub async fn handler(
    database: &Database,
    user_id: Uuid,
    request: Request,
) -> Result<Response, Error> {
    let playlist_id = request.playlist_id;
    playlist::permission::check_write(database, playlist_id, user_id, false).await?;

    if request.name.is_some() || request.comment.is_some() || request.public.is_some() {
        playlists::Upsert::from(&request).update(database, playlist_id).await?;
    }

    if let Some(song_indexes) = request.remove_indexes {
        // TODO: Do it in one query.
        let song_ids = playlists_songs::table
            .left_join(songs::table)
            .left_join(albums::table.on(albums::id.eq(songs::album_id)))
            .filter(playlists_songs::playlist_id.eq(playlist_id))
            .filter(playlist::query::with_album(user_id))
            .select(playlists_songs::song_id)
            .order_by(playlists_songs::created_at)
            .get_results::<Uuid>(&mut database.get().await?)
            .await?;

        let song_ids: Vec<_> = song_indexes
            .into_iter()
            .filter_map(|index| song_ids.get::<usize>(index.into()))
            .collect();

        diesel::delete(playlists_songs::table)
            .filter(playlists_songs::playlist_id.eq(playlist_id))
            .filter(playlists_songs::song_id.eq_any(song_ids))
            .execute(&mut database.get().await?)
            .await?;
    }

    if let Some(song_ids) = request.add_ids {
        playlists_songs::Upsert::upserts(database, playlist_id, &song_ids).await?;
    }

    Ok(Response)
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};
    use rstest::rstest;

    use super::*;
    use crate::route::playlists::{create_playlist, get_playlist};
    use crate::test::{mock, Mock};

    #[rstest]
    #[case(0, false, &[], &[])]
    #[case(0, true, &[], &[])]
    #[case(5, false, &[], &[5, 6, 7, 8, 9])]
    #[case(5, true, &[], &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9])]
    #[case(5, false, &[0, 1, 5, 9], &[7, 8, 9])]
    #[case(5, true, &[0, 1, 5, 9], &[2, 3, 4, 6, 7, 8])]
    #[case(5, false, &[5, 1, 0, 9], &[7, 8, 9])]
    #[case(5, true, &[5, 1, 0, 9], &[2, 3, 4, 6, 7, 8])]
    #[case(5, false, &[0, 1, 5, 9, 19], &[7, 8, 9])]
    #[case(5, true, &[0, 1, 5, 9, 19], &[2, 3, 4, 6, 7, 8])]
    #[tokio::test]
    async fn test_delete(
        #[future(awt)]
        #[with(1, 0)]
        mock: Mock,
        #[case] n_song: usize,
        #[case] allow: bool,
        #[case] remove_indexes: &[u16],
        #[case] retain_indexes: &[u16],
    ) {
        mock.add_music_folder().allow(allow).call().await;
        mock.add_music_folder().call().await;

        let mut music_folder_permission = mock.music_folder(0).await;
        let mut music_folder = mock.music_folder(1).await;

        music_folder_permission.add_audio().n_song(n_song).call().await;
        music_folder.add_audio().n_song(n_song).call().await;

        let song_ids: Vec<_> = music_folder_permission
            .database
            .keys()
            .copied()
            .chain(music_folder.database.keys().copied())
            .collect();

        let user_id = mock.user_id(0).await;
        let playlist_id = create_playlist::handler(
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
        .id;

        handler(
            mock.database(),
            user_id,
            Request {
                playlist_id,
                remove_indexes: Some(remove_indexes.into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();

        let song_ids: Vec<_> = retain_indexes
            .iter()
            .filter_map(|index| song_ids.get::<usize>((*index).into()))
            .copied()
            .collect();
        let database_song_ids: Vec<_> = get_playlist::handler(
            mock.database(),
            user_id,
            get_playlist::Request { id: playlist_id },
        )
        .await
        .unwrap()
        .playlist
        .entry
        .into_iter()
        .map(|entry| entry.song.id)
        .collect();
        assert_eq!(database_song_ids, song_ids);
    }
}
