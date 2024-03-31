use anyhow::Result;
use axum::extract::State;
use diesel::dsl::{count_distinct, sql};
use diesel::{sql_types, ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_validate, wrap_subsonic_response};
use serde::Serialize;
use uuid::Uuid;

use crate::models::*;
use crate::open_subsonic::common::error::OSError;
use crate::open_subsonic::common::id3::db::*;
use crate::open_subsonic::common::id3::response::*;
use crate::open_subsonic::common::music_folder::with_music_folders;
use crate::{Database, DatabasePool};

#[add_validate]
#[derive(Debug)]
pub struct GetAlbumParams {
    id: Uuid,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumId3WithSongs {
    #[serde(flatten)]
    pub album: AlbumId3,
    #[serde(rename = "song")]
    pub songs: Vec<SongId3>,
}

#[wrap_subsonic_response]
pub struct GetAlbumBody {
    album: AlbumId3WithSongs,
}

async fn get_album_and_song_ids(
    pool: &DatabasePool,
    user_id: Uuid,
    album_id: Uuid,
) -> Result<(AlbumId3Db, Vec<Uuid>)> {
    songs::table
        .inner_join(albums::table)
        .inner_join(songs_album_artists::table)
        .filter(with_music_folders(user_id))
        .filter(albums::id.eq(album_id))
        .group_by(albums::id)
        .having(count_distinct(songs::id).gt(0))
        .select((
            AlbumId3Db::as_select(),
            sql::<sql_types::Array<sql_types::Uuid>>("array_agg(distinct(songs.id)) song_ids"),
        ))
        .first::<(AlbumId3Db, Vec<Uuid>)>(&mut pool.get().await?)
        .await
        .optional()?
        .ok_or_else(|| OSError::NotFound("Album".into()).into())
}

async fn get_basic_songs(
    pool: &DatabasePool,
    user_id: Uuid,
    song_ids: &[Uuid],
) -> Result<Vec<BasicSongId3Db>> {
    songs::table
        .filter(with_music_folders(user_id))
        .filter(songs::id.eq_any(song_ids))
        .select(BasicSongId3Db::as_select())
        .get_results::<BasicSongId3Db>(&mut pool.get().await?)
        .await
        .map_err(anyhow::Error::from)
}

pub async fn get_album(
    pool: &DatabasePool,
    user_id: Uuid,
    album_id: Uuid,
) -> Result<AlbumId3WithSongs> {
    let (album, song_ids) = get_album_and_song_ids(pool, user_id, album_id).await?;
    let basic_songs = get_basic_songs(pool, user_id, &song_ids).await?;

    Ok(AlbumId3WithSongs {
        album: album.into_res(pool).await?,
        songs: basic_songs.into_iter().map(BasicSongId3Db::into_res).collect(),
    })
}

pub async fn get_album_handler(
    State(database): State<Database>,
    req: GetAlbumRequest,
) -> GetAlbumJsonResponse {
    GetAlbumBody { album: get_album(&database.pool, req.user_id, req.params.id).await? }.into()
}

#[cfg(test)]
mod tests {
    use diesel::JoinOnDsl;
    use fake::{Fake, Faker};
    use itertools::Itertools;
    use rand::Rng;

    use super::*;
    use crate::open_subsonic::scan::test::upsert_album;
    use crate::utils::song::test::SongTag;
    use crate::utils::test::Infra;

    async fn get_artist_ids(pool: &DatabasePool, user_id: Uuid, album_id: Uuid) -> Vec<Uuid> {
        songs::table
            .inner_join(albums::table)
            .inner_join(songs_album_artists::table)
            .inner_join(artists::table.on(artists::id.eq(songs_album_artists::album_artist_id)))
            .filter(with_music_folders(user_id))
            .filter(albums::id.eq(album_id))
            .select(artists::id)
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
                (0..n_song)
                    .map(|_| SongTag { album: album_name.to_owned(), ..Faker.fake() })
                    .collect(),
            )
            .scan(.., None)
            .await;

        let album_id = upsert_album(infra.pool(), album_name.into()).await.unwrap();
        let album_id3 =
            get_album_and_song_ids(infra.pool(), infra.user_id(0), album_id).await.unwrap().0;
        let artist_ids = get_artist_ids(infra.pool(), infra.user_id(0), album_id).await;

        assert_eq!(album_id3.basic.id, album_id);
        assert_eq!(album_id3.basic.name, album_name);
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
                        album: album_name.to_owned(),
                        album_artists: if i < 5 {
                            vec![artist_names[0].to_owned()]
                        } else {
                            vec![artist_names[1].to_owned()]
                        },
                        ..Faker.fake()
                    })
                    .collect(),
            )
            .scan(.., None)
            .await;

        let album_id = upsert_album(infra.pool(), album_name.into()).await.unwrap();
        let album_id3 =
            get_album_and_song_ids(infra.pool(), infra.user_id(0), album_id).await.unwrap().0;
        let artist_ids = get_artist_ids(infra.pool(), infra.user_id(0), album_id).await;

        assert_eq!(album_id3.artist_ids.clone().into_iter().sorted().collect_vec(), artist_ids);
        assert_eq!(
            album_id3
                .into_res(infra.pool())
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
                (0..n_song)
                    .map(|_| SongTag { album: album_name.to_owned(), ..Faker.fake() })
                    .collect(),
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
                (0..n_song)
                    .map(|_| SongTag { album: album_name.to_owned(), ..Faker.fake() })
                    .collect(),
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
                (0..n_song)
                    .map(|_| SongTag { album: album_name.to_owned(), ..Faker.fake() })
                    .collect(),
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
