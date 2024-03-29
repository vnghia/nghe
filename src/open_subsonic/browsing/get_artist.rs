use anyhow::Result;
use axum::extract::State;
use diesel::dsl::{count_distinct, sql};
use diesel::{
    sql_types, BoolExpressionMethods, ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl,
    SelectableHelper,
};
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_validate, wrap_subsonic_response};
use serde::Serialize;
use uuid::Uuid;

use crate::models::*;
use crate::open_subsonic::common::id3::db::*;
use crate::open_subsonic::common::id3::response::*;
use crate::open_subsonic::common::music_folder::check_user_music_folder_ids;
use crate::{Database, DatabasePool, OSError};

#[add_validate]
#[derive(Debug)]
pub struct GetArtistParams {
    id: Uuid,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistId3WithAlbums {
    #[serde(flatten)]
    pub artist: ArtistId3,
    #[serde(rename = "album")]
    pub albums: Vec<AlbumId3>,
}

#[wrap_subsonic_response]
pub struct GetArtistBody {
    artist: ArtistId3WithAlbums,
}

async fn get_artist_and_album_ids(
    pool: &DatabasePool,
    music_folder_ids: &[Uuid],
    artist_id: &Uuid,
) -> Result<(ArtistId3Db, Vec<Uuid>)> {
    artists::table
        .left_join(songs_album_artists::table)
        .left_join(songs_artists::table)
        .inner_join(songs::table.on(
            songs::id.eq(songs_album_artists::song_id).or(songs::id.eq(songs_artists::song_id)),
        ))
        .filter(songs::music_folder_id.eq_any(music_folder_ids))
        .filter(artists::id.eq(artist_id))
        .group_by(artists::id)
        .having(count_distinct(songs::album_id).gt(0))
        .select((
            ArtistId3Db::as_select(),
            sql::<sql_types::Array<sql_types::Uuid>>(
                "array_agg(distinct(songs.album_id)) album_ids",
            ),
        ))
        .first::<(ArtistId3Db, Vec<Uuid>)>(&mut pool.get().await?)
        .await
        .optional()?
        .ok_or_else(|| OSError::NotFound("Artist".into()).into())
}

async fn get_basic_albums(
    pool: &DatabasePool,
    music_folder_ids: &[Uuid],
    album_ids: &[Uuid],
) -> Result<Vec<BasicAlbumId3Db>> {
    albums::table
        .inner_join(songs::table)
        .filter(songs::music_folder_id.eq_any(music_folder_ids))
        .filter(albums::id.eq_any(album_ids))
        .group_by(albums::id)
        .select(BasicAlbumId3Db::as_select())
        .get_results::<BasicAlbumId3Db>(&mut pool.get().await?)
        .await
        .map_err(anyhow::Error::from)
}

pub async fn get_artist(
    pool: &DatabasePool,
    user_id: Uuid,
    artist_id: Uuid,
) -> Result<ArtistId3WithAlbums> {
    let music_folder_ids = check_user_music_folder_ids(pool, &user_id, None).await?;

    let (artist, album_ids) = get_artist_and_album_ids(pool, &music_folder_ids, &artist_id).await?;
    let basic_albums = get_basic_albums(pool, &music_folder_ids, &album_ids).await?;

    Ok(ArtistId3WithAlbums {
        artist: artist.into_res(),
        albums: basic_albums.into_iter().map(|v| v.into_res()).collect(),
    })
}

pub async fn get_artist_handler(
    State(database): State<Database>,
    req: GetArtistRequest,
) -> GetArtistJsonResponse {
    GetArtistBody { artist: get_artist(&database.pool, req.user_id, req.params.id).await? }.into()
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};
    use itertools::Itertools;
    use rand::Rng;

    use super::*;
    use crate::open_subsonic::scan::test::upsert_artists;
    use crate::utils::song::test::SongTag;
    use crate::utils::test::media::song_paths_to_album_ids;
    use crate::utils::test::setup::TestInfra;

    #[tokio::test]
    async fn test_get_artist_own_albums() {
        let artist_name = "artist";
        let n_song = 10_usize;

        let (test_infra, song_fs_infos) = TestInfra::setup_songs(
            &[n_song],
            (0..n_song)
                .map(|_| SongTag { album_artists: vec![artist_name.to_owned()], ..Faker.fake() })
                .collect_vec(),
        )
        .await;

        let artist_id = upsert_artists(test_infra.pool(), &[artist_name]).await.unwrap().remove(0);
        let music_folder_ids = test_infra.music_folder_ids(..);
        let album_fs_ids = song_paths_to_album_ids(test_infra.pool(), &song_fs_infos).await;

        let album_ids = get_artist_and_album_ids(test_infra.pool(), &music_folder_ids, &artist_id)
            .await
            .unwrap()
            .1
            .into_iter()
            .sorted()
            .collect_vec();

        assert_eq!(album_ids, album_fs_ids);
    }

    #[tokio::test]
    async fn test_get_artist_featured_in_albums() {
        let artist_name = "artist";
        let n_song = 10_usize;

        let (test_infra, song_fs_infos) = TestInfra::setup_songs(
            &[n_song],
            (0..n_song)
                .map(|_| SongTag { artists: vec![artist_name.to_owned()], ..Faker.fake() })
                .collect_vec(),
        )
        .await;

        let artist_id = upsert_artists(test_infra.pool(), &[artist_name]).await.unwrap().remove(0);
        let music_folder_ids = test_infra.music_folder_ids(..);
        let album_fs_ids = song_paths_to_album_ids(test_infra.pool(), &song_fs_infos).await;

        let album_ids = get_artist_and_album_ids(test_infra.pool(), &music_folder_ids, &artist_id)
            .await
            .unwrap()
            .1
            .into_iter()
            .sorted()
            .collect_vec();

        assert_eq!(album_ids, album_fs_ids);
    }

    #[tokio::test]
    async fn test_get_artist_distinct_albums() {
        let artist_name = "artist";
        let album_names = ["album1", "album2"];
        let n_song = 10_usize;

        let (test_infra, song_fs_infos) = TestInfra::setup_songs(
            &[n_song],
            (0..n_song)
                .map(|i| SongTag {
                    artists: vec![artist_name.to_owned()],
                    album: if i < 5 {
                        album_names[0].to_owned()
                    } else {
                        album_names[1].to_owned()
                    },
                    ..Faker.fake()
                })
                .collect_vec(),
        )
        .await;

        let artist_id = upsert_artists(test_infra.pool(), &[artist_name]).await.unwrap().remove(0);
        let music_folder_ids = test_infra.music_folder_ids(..);
        let album_fs_ids = song_paths_to_album_ids(test_infra.pool(), &song_fs_infos).await;

        let album_ids = get_artist_and_album_ids(test_infra.pool(), &music_folder_ids, &artist_id)
            .await
            .unwrap()
            .1
            .into_iter()
            .sorted()
            .collect_vec();

        assert_eq!(album_ids.len(), album_names.len());
        assert_eq!(album_ids, album_fs_ids);
    }

    #[tokio::test]
    async fn test_get_artist_albums_partial_music_folders() {
        let artist_name = "artist";
        let n_folder = 2_usize;
        let n_song = 10_usize;

        let (test_infra, song_fs_infos) = TestInfra::setup_songs(
            &vec![n_song; n_folder],
            (0..n_folder * n_song)
                .map(|_| SongTag { artists: vec![artist_name.to_owned()], ..Faker.fake() })
                .collect_vec(),
        )
        .await;

        let artist_id = upsert_artists(test_infra.pool(), &[artist_name]).await.unwrap().remove(0);
        let music_folder_idx = rand::thread_rng().gen_range(0..test_infra.music_folders.len());
        let music_folder_ids = test_infra.music_folder_ids(music_folder_idx..=music_folder_idx);
        let album_fs_ids = song_paths_to_album_ids(
            test_infra.pool(),
            &song_fs_infos[music_folder_idx * n_song..(music_folder_idx + 1) * n_song],
        )
        .await;

        let album_ids = get_artist_and_album_ids(test_infra.pool(), &music_folder_ids, &artist_id)
            .await
            .unwrap()
            .1
            .into_iter()
            .sorted()
            .collect_vec();

        assert_eq!(album_ids, album_fs_ids);
    }

    #[tokio::test]
    async fn test_get_artist_albums_deny_music_folders() {
        let artist_name = "artist";
        let n_folder = 2_usize;
        let n_scan_folder = 1_usize;
        let n_song = 10_usize;

        let (test_infra, _) = TestInfra::setup_songs(
            &vec![n_song; n_folder],
            (0..n_folder * n_song)
                .map(|i| SongTag {
                    artists: if i >= n_scan_folder * n_song {
                        vec![artist_name.to_owned()]
                    } else {
                        fake::vec![String; 1..2]
                    },
                    ..Faker.fake()
                })
                .collect_vec(),
        )
        .await;

        let artist_id = upsert_artists(test_infra.pool(), &[artist_name]).await.unwrap().remove(0);
        let music_folder_ids = test_infra.music_folder_ids(..);

        assert!(matches!(
            get_artist_and_album_ids(
                test_infra.pool(),
                &music_folder_ids[..n_scan_folder],
                &artist_id,
            )
            .await
            .unwrap_err()
            .root_cause()
            .downcast_ref::<OSError>()
            .unwrap(),
            OSError::NotFound(_)
        ));
    }

    #[tokio::test]
    async fn test_get_basic_albums() {
        let album_names = ["album1", "album2"];
        let n_folder = 3_usize;
        let n_song = 10_usize;
        let n_diff = 3_usize;

        let (test_infra, song_fs_infos) = TestInfra::setup_songs(
            &[n_song, n_song + n_diff, n_song - n_diff],
            (0..n_folder * n_song)
                .map(|i| SongTag {
                    album: if i < n_song {
                        album_names[0].to_owned()
                    } else {
                        album_names[1].to_owned()
                    },
                    ..Faker.fake()
                })
                .collect_vec(),
        )
        .await;

        let music_folder_ids = test_infra.music_folder_ids(..2);
        let album_fs_ids = song_paths_to_album_ids(test_infra.pool(), &song_fs_infos).await;

        let basic_albums = get_basic_albums(test_infra.pool(), &music_folder_ids, &album_fs_ids)
            .await
            .unwrap()
            .into_iter()
            .sorted_by(|a, b| a.name.cmp(&b.name))
            .collect_vec();

        assert_eq!(basic_albums[0].song_count as usize, n_song);
        assert_eq!(basic_albums[1].song_count as usize, n_song + n_diff);
    }
}
