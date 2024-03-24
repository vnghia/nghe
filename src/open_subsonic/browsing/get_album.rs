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
use crate::open_subsonic::common::music_folder::check_user_music_folder_ids;
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
    pub songs: Vec<ChildId3>,
}

#[wrap_subsonic_response]
pub struct GetAlbumBody {
    album: AlbumId3WithSongs,
}

async fn get_album_and_song_ids(
    pool: &DatabasePool,
    music_folder_ids: &[Uuid],
    album_id: &Uuid,
) -> Result<(AlbumId3Db, Vec<Uuid>)> {
    songs::table
        .inner_join(albums::table)
        .inner_join(songs_album_artists::table)
        .filter(songs::music_folder_id.eq_any(music_folder_ids))
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
    music_folder_ids: &[Uuid],
    song_ids: &[Uuid],
) -> Result<Vec<BasicChildId3Db>> {
    songs::table
        .filter(songs::music_folder_id.eq_any(music_folder_ids))
        .filter(songs::id.eq_any(song_ids))
        .select(BasicChildId3Db::as_select())
        .get_results::<BasicChildId3Db>(&mut pool.get().await?)
        .await
        .map_err(anyhow::Error::from)
}

pub async fn get_album(
    pool: &DatabasePool,
    user_id: Uuid,
    album_id: Uuid,
) -> Result<AlbumId3WithSongs> {
    let music_folder_ids = check_user_music_folder_ids(pool, &user_id, None).await?;

    let (album, song_ids) = get_album_and_song_ids(pool, &music_folder_ids, &album_id).await?;
    let basic_songs = get_basic_songs(pool, &music_folder_ids, &song_ids).await?;

    Ok(AlbumId3WithSongs {
        album: album.into_res(pool).await?,
        songs: basic_songs.into_iter().map(BasicChildId3Db::into_res).collect(),
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
    use crate::utils::test::media::song_paths_to_ids;
    use crate::utils::test::setup::TestInfra;

    async fn get_artist_ids(
        pool: &DatabasePool,
        music_folder_ids: &[Uuid],
        album_id: &Uuid,
    ) -> Vec<Uuid> {
        songs::table
            .inner_join(albums::table)
            .inner_join(songs_album_artists::table)
            .inner_join(artists::table.on(artists::id.eq(songs_album_artists::album_artist_id)))
            .filter(songs::music_folder_id.eq_any(music_folder_ids))
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

        let (test_infra, _) = TestInfra::setup_songs(
            &[n_song],
            (0..n_song)
                .map(|_| SongTag { album: album_name.to_owned(), ..Faker.fake() })
                .collect_vec(),
        )
        .await;

        let album_id = upsert_album(test_infra.pool(), album_name.into()).await.unwrap();
        let music_folder_ids = test_infra.music_folder_ids(..);

        let album_id3 = get_album_and_song_ids(test_infra.pool(), &music_folder_ids, &album_id)
            .await
            .unwrap()
            .0;
        let artist_ids = get_artist_ids(test_infra.pool(), &music_folder_ids, &album_id).await;

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

        let (test_infra, _) = TestInfra::setup_songs(
            &[n_song],
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
                .collect_vec(),
        )
        .await;

        let album_id = upsert_album(test_infra.pool(), album_name.into()).await.unwrap();
        let music_folder_ids = test_infra.music_folder_ids(..);

        let album_id3 = get_album_and_song_ids(test_infra.pool(), &music_folder_ids, &album_id)
            .await
            .unwrap()
            .0;
        let artist_ids = get_artist_ids(test_infra.pool(), &music_folder_ids, &album_id).await;

        assert_eq!(album_id3.artist_ids.clone().into_iter().sorted().collect_vec(), artist_ids);
        assert_eq!(
            album_id3
                .into_res(test_infra.pool())
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

        let (test_infra, song_fs_infos) = TestInfra::setup_songs(
            &[n_song],
            (0..n_song)
                .map(|_| SongTag { album: album_name.to_owned(), ..Faker.fake() })
                .collect_vec(),
        )
        .await;

        let album_id = upsert_album(test_infra.pool(), album_name.into()).await.unwrap();
        let music_folder_ids = test_infra.music_folder_ids(..);

        let song_ids = get_album_and_song_ids(test_infra.pool(), &music_folder_ids, &album_id)
            .await
            .unwrap()
            .1
            .into_iter()
            .sorted()
            .collect_vec();
        let song_fs_ids = song_paths_to_ids(test_infra.pool(), &song_fs_infos).await;

        assert_eq!(song_ids, song_fs_ids);
    }

    #[tokio::test]
    async fn test_get_album_song_ids_partial_music_folders() {
        let album_name = "album";
        let n_folder = 2_usize;
        let n_song = 10_usize;

        let (test_infra, song_fs_infos) = TestInfra::setup_songs(
            &vec![n_song; n_folder],
            (0..n_folder * n_song)
                .map(|_| SongTag { album: album_name.to_owned(), ..Faker.fake() })
                .collect_vec(),
        )
        .await;

        let album_id = upsert_album(test_infra.pool(), album_name.into()).await.unwrap();
        let music_folder_idx = rand::thread_rng().gen_range(0..test_infra.music_folders.len());
        let music_folder_ids = test_infra.music_folder_ids(music_folder_idx..=music_folder_idx);

        let song_ids = get_album_and_song_ids(test_infra.pool(), &music_folder_ids, &album_id)
            .await
            .unwrap()
            .1
            .into_iter()
            .sorted()
            .collect_vec();
        let song_fs_ids = song_paths_to_ids(
            test_infra.pool(),
            &song_fs_infos[music_folder_idx * n_song..(music_folder_idx + 1) * n_song],
        )
        .await;

        assert_eq!(song_ids, song_fs_ids);
    }

    #[tokio::test]
    async fn test_get_album_song_ids_deny_music_folders() {
        let album_name = "album";
        let n_folder = 2_usize;
        let n_scan_folder = 1_usize;
        let n_song = 10_usize;

        let (test_infra, _) = TestInfra::setup_songs(
            &vec![n_song; n_folder],
            (0..n_folder * n_song)
                .map(|i| SongTag {
                    album: if i >= n_scan_folder * n_song {
                        album_name.to_owned()
                    } else {
                        Faker.fake::<String>()
                    },
                    ..Faker.fake()
                })
                .collect_vec(),
        )
        .await;
        let music_folder_ids = test_infra.music_folder_ids(..);
        let album_id = upsert_album(test_infra.pool(), album_name.into()).await.unwrap();

        assert!(matches!(
            get_album_and_song_ids(
                test_infra.pool(),
                &music_folder_ids[..n_scan_folder],
                &album_id,
            )
            .await
            .unwrap_err()
            .root_cause()
            .downcast_ref::<OSError>()
            .unwrap(),
            OSError::NotFound(_)
        ));
    }
}
