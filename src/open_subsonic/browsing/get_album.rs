use anyhow::Result;
use axum::extract::State;
use diesel::dsl::sql;
use diesel::{sql_types, ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use futures::{stream, StreamExt, TryStreamExt};
use nghe_proc_macros::{add_axum_response, add_common_validate};
use uuid::Uuid;

use crate::models::*;
use crate::open_subsonic::common::error::OSError;
use crate::open_subsonic::id3::*;
use crate::open_subsonic::permission::with_permission;
use crate::{Database, DatabasePool};

add_common_validate!(GetAlbumParams);
add_axum_response!(GetAlbumBody);

async fn get_album_and_song_ids(
    pool: &DatabasePool,
    user_id: Uuid,
    album_id: Uuid,
) -> Result<(AlbumId3Db, Vec<Uuid>)> {
    get_album_id3_db()
        .filter(with_permission(user_id))
        .filter(albums::id.eq(album_id))
        .select((
            AlbumId3Db::as_select(),
            sql::<sql_types::Array<sql_types::Uuid>>("array_agg(distinct(songs.id)) song_ids"),
        ))
        .first::<(AlbumId3Db, Vec<Uuid>)>(&mut pool.get().await?)
        .await
        .optional()?
        .ok_or_else(|| OSError::NotFound("Album".into()).into())
}

async fn get_songs(
    pool: &DatabasePool,
    user_id: Uuid,
    song_ids: &[Uuid],
) -> Result<Vec<SongId3Db>> {
    get_song_id3_db()
        .filter(with_permission(user_id))
        .filter(songs::id.eq_any(song_ids))
        .get_results::<SongId3Db>(&mut pool.get().await?)
        .await
        .map_err(anyhow::Error::from)
}

pub async fn get_album(
    pool: &DatabasePool,
    user_id: Uuid,
    album_id: Uuid,
) -> Result<AlbumId3WithSongs> {
    let (album, song_ids) = get_album_and_song_ids(pool, user_id, album_id).await?;
    let songs = get_songs(pool, user_id, &song_ids).await?;

    Ok(AlbumId3WithSongs {
        album: album.into(pool).await?,
        songs: stream::iter(songs)
            .then(|v| async move { v.into(pool).await })
            .try_collect()
            .await?,
    })
}

pub async fn get_album_handler(
    State(database): State<Database>,
    req: GetAlbumRequest,
) -> GetAlbumJsonResponse {
    Ok(axum::Json(
        GetAlbumBody { album: get_album(&database.pool, req.user_id, req.params.id).await? }.into(),
    ))
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};
    use itertools::Itertools;
    use rand::Rng;

    use super::*;
    use crate::open_subsonic::scan::test::upsert_album;
    use crate::utils::song::test::SongTag;
    use crate::utils::test::Infra;

    async fn get_artist_ids(pool: &DatabasePool, user_id: Uuid, album_id: Uuid) -> Vec<Uuid> {
        Infra::get_album_artist_db()
            .filter(with_permission(user_id))
            .filter(songs::album_id.eq(album_id))
            .select(artists::id)
            .distinct()
            .get_results::<Uuid>(&mut pool.get().await.unwrap())
            .await
            .unwrap()
            .into_iter()
            .unique()
            .sorted()
            .collect_vec()
    }

    #[tokio::test]
    async fn test_get_album_id3() {
        let album_name = "album";
        let n_song = 10_usize;
        let mut infra = Infra::new().await.n_folder(1).await.add_user(None).await;
        infra
            .add_songs(
                0,
                (0..n_song).map(|_| SongTag { album: album_name.into(), ..Faker.fake() }).collect(),
            )
            .scan(.., None)
            .await;

        let album_id = upsert_album(infra.pool(), album_name.into()).await.unwrap();
        let album_id3 =
            get_album_and_song_ids(infra.pool(), infra.user_id(0), album_id).await.unwrap().0;
        let artist_ids = get_artist_ids(infra.pool(), infra.user_id(0), album_id).await;

        assert_eq!(album_id3.basic.id, album_id);
        assert_eq!(album_id3.basic.no_id.name, album_name);
        assert_eq!(album_id3.basic.song_count as usize, n_song);
        assert_eq!(album_id3.artist_ids.into_iter().sorted().collect_vec(), artist_ids);
    }

    #[tokio::test]
    async fn test_get_album_distinct_basic_artists() {
        let album_name = "album";
        let artist_names = ["artist1", "artist2"];
        let n_song = 10_usize;
        let mut infra = Infra::new().await.n_folder(1).await.add_user(None).await;
        infra
            .add_songs(
                0,
                (0..n_song)
                    .map(|i| SongTag {
                        album: album_name.into(),
                        album_artists: if i < 5 {
                            vec![artist_names[0].into()]
                        } else {
                            vec![artist_names[1].into()]
                        },
                        ..Faker.fake()
                    })
                    .collect(),
            )
            .scan(.., None)
            .await;

        let album_id = upsert_album(infra.pool(), album_name.into()).await.unwrap();
        let album_media: crate::utils::song::MediaDateMbz = album_name.into();
        println!("{:?}", album_media);
        let album_id3 =
            get_album_and_song_ids(infra.pool(), infra.user_id(0), album_id).await.unwrap().0;
        let artist_ids = get_artist_ids(infra.pool(), infra.user_id(0), album_id).await;

        assert_eq!(album_id3.artist_ids.clone().into_iter().sorted().collect_vec(), artist_ids);
        assert_eq!(
            album_id3
                .into(infra.pool())
                .await
                .unwrap()
                .artists
                .into_iter()
                .map(|v| v.name)
                .sorted()
                .collect_vec(),
            artist_names
        );
    }

    #[tokio::test]
    async fn test_get_album_song_ids() {
        let album_name = "album";
        let n_song = 10_usize;
        let mut infra = Infra::new().await.n_folder(1).await.add_user(None).await;
        infra
            .add_songs(
                0,
                (0..n_song).map(|_| SongTag { album: album_name.into(), ..Faker.fake() }).collect(),
            )
            .scan(.., None)
            .await;

        let album_id = upsert_album(infra.pool(), album_name.into()).await.unwrap();
        let song_ids = get_album_and_song_ids(infra.pool(), infra.user_id(0), album_id)
            .await
            .unwrap()
            .1
            .into_iter()
            .sorted()
            .collect_vec();
        assert_eq!(song_ids, infra.song_ids(..).await);
    }

    #[tokio::test]
    async fn test_get_album_song_ids_partial_music_folders() {
        let album_name = "album";
        let n_folder = 2_usize;
        let n_song = 10_usize;
        let mut infra = Infra::new().await.n_folder(n_folder).await.add_user(None).await;
        (0..n_folder).for_each(|i| {
            infra.add_songs(
                i,
                (0..n_song).map(|_| SongTag { album: album_name.into(), ..Faker.fake() }).collect(),
            );
        });
        infra.scan(.., None).await;

        let album_id = upsert_album(infra.pool(), album_name.into()).await.unwrap();
        let music_folder_idx = rand::thread_rng().gen_range(0..infra.music_folders.len());
        infra.only_permissions(.., music_folder_idx..=music_folder_idx, true).await;

        let song_ids = get_album_and_song_ids(infra.pool(), infra.user_id(0), album_id)
            .await
            .unwrap()
            .1
            .into_iter()
            .sorted()
            .collect_vec();
        assert_eq!(song_ids, infra.song_ids(music_folder_idx..=music_folder_idx).await);
    }

    #[tokio::test]
    async fn test_get_album_song_ids_deny_music_folders() {
        // The album exists but does not reside in the allowed music folders.
        let album_name = "album";
        let n_folder = 2_usize;
        let n_scan_folder = 1_usize;
        let n_song = 10_usize;
        let mut infra = Infra::new().await.n_folder(n_folder).await.add_user(None).await;
        infra
            .add_songs(0, (0..n_song).map(|_| Faker.fake()).collect())
            .add_songs(
                1,
                (0..n_song).map(|_| SongTag { album: album_name.into(), ..Faker.fake() }).collect(),
            )
            .scan(.., None)
            .await;
        infra.permissions(.., n_scan_folder.., false).await;

        let album_id = upsert_album(infra.pool(), album_name.into()).await.unwrap();

        assert!(matches!(
            get_album_and_song_ids(infra.pool(), infra.user_id(0), album_id)
                .await
                .unwrap_err()
                .root_cause()
                .downcast_ref::<OSError>()
                .unwrap(),
            OSError::NotFound(_)
        ));
    }
}
