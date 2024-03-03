use crate::{
    models::*,
    open_subsonic::common::{
        id3::{ArtistId3, BasicAlbumId3},
        music_folder::check_user_music_folder_ids,
    },
    Database, DatabasePool, OSError,
};

use anyhow::Result;
use axum::extract::State;
use diesel::{
    dsl::{count, count_distinct, sql, sum},
    sql_types, BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods,
    OptionalExtension, QueryDsl,
};
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_validate, wrap_subsonic_response};
use serde::Serialize;
use uuid::Uuid;

#[add_validate]
#[derive(Debug)]
pub struct GetArtistParams {
    id: Uuid,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistId3WithAlbums {
    #[serde(flatten)]
    artist: ArtistId3,
    album: Vec<BasicAlbumId3>,
}

#[wrap_subsonic_response]
pub struct GetArtistBody {
    artist: ArtistId3WithAlbums,
}

async fn get_artist_and_album_ids(
    pool: &DatabasePool,
    music_folder_ids: &[Uuid],
    artist_id: &Uuid,
) -> Result<(ArtistId3, Vec<Uuid>)> {
    artists::table
        .left_join(songs_album_artists::table)
        .left_join(songs_artists::table)
        .inner_join(
            songs::table.on(songs::id
                .eq(songs_album_artists::song_id)
                .or(songs::id.eq(songs_artists::song_id))),
        )
        .filter(songs::music_folder_id.eq_any(music_folder_ids))
        .filter(artists::id.eq(artist_id))
        .group_by(artists::id)
        .having(count_distinct(songs::album_id).gt(0))
        .select((
            ((artists::id, artists::name),),
            sql::<sql_types::Array<sql_types::Uuid>>(
                "array_agg(distinct songs.album_id) album_ids",
            ),
        ))
        .first::<(ArtistId3, Vec<Uuid>)>(&mut pool.get().await?)
        .await
        .optional()?
        .ok_or_else(|| OSError::NotFound("Artist".into()).into())
}

async fn get_basic_albums(
    pool: &DatabasePool,
    music_folder_ids: &[Uuid],
    album_ids: &[Uuid],
) -> Result<Vec<BasicAlbumId3>> {
    albums::table
        .inner_join(songs::table)
        .filter(songs::music_folder_id.eq_any(music_folder_ids))
        .filter(albums::id.eq_any(album_ids))
        .group_by(albums::id)
        .select((
            albums::id,
            albums::name,
            count(songs::id),
            sum(songs::duration).assume_not_null(),
            albums::created_at,
        ))
        .get_results::<BasicAlbumId3>(&mut pool.get().await?)
        .await
        .map_err(anyhow::Error::from)
}

pub async fn get_artist_handler(
    State(database): State<Database>,
    req: GetArtistRequest,
) -> GetArtistJsonResponse {
    let music_folder_ids = check_user_music_folder_ids(&database.pool, &req.user.id, None).await?;

    let (artist, album_ids) =
        get_artist_and_album_ids(&database.pool, &music_folder_ids, &req.params.id).await?;
    let basic_albums = get_basic_albums(&database.pool, &music_folder_ids, &album_ids).await?;

    GetArtistBody {
        artist: ArtistId3WithAlbums {
            artist,
            album: basic_albums,
        },
    }
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        open_subsonic::scan::tests::upsert_artists,
        utils::{
            song::tag::SongTag,
            test::{media::song_paths_to_album_ids, setup::setup_songs},
        },
    };

    use fake::{Fake, Faker};
    use itertools::Itertools;
    use rand::seq::SliceRandom;

    #[tokio::test]
    async fn test_get_artist_own_albums() {
        let artist_name = "artist";
        let n_song = 10_usize;

        let (temp_db, _temp_fs, music_folders, song_fs_info) = setup_songs(
            1,
            &[n_song],
            (0..n_song)
                .map(|_| SongTag {
                    album_artists: vec![artist_name.to_owned()],
                    ..Faker.fake()
                })
                .collect_vec(),
        )
        .await;

        let artist_id = upsert_artists(temp_db.pool(), &[artist_name])
            .await
            .unwrap()
            .remove(0);
        let music_folder_ids = music_folders
            .iter()
            .map(|music_folder| music_folder.id)
            .collect_vec();
        let album_fs_ids = song_paths_to_album_ids(temp_db.pool(), &song_fs_info).await;

        let album_ids = get_artist_and_album_ids(temp_db.pool(), &music_folder_ids, &artist_id)
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

        let (temp_db, _temp_fs, music_folders, song_fs_info) = setup_songs(
            1,
            &[n_song],
            (0..n_song)
                .map(|_| SongTag {
                    artists: vec![artist_name.to_owned()],
                    ..Faker.fake()
                })
                .collect_vec(),
        )
        .await;

        let artist_id = upsert_artists(temp_db.pool(), &[artist_name])
            .await
            .unwrap()
            .remove(0);
        let music_folder_ids = music_folders
            .iter()
            .map(|music_folder| music_folder.id)
            .collect_vec();
        let album_fs_ids = song_paths_to_album_ids(temp_db.pool(), &song_fs_info).await;

        let album_ids = get_artist_and_album_ids(temp_db.pool(), &music_folder_ids, &artist_id)
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

        let (temp_db, _temp_fs, music_folders, song_fs_info) = setup_songs(
            1,
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

        let artist_id = upsert_artists(temp_db.pool(), &[artist_name])
            .await
            .unwrap()
            .remove(0);
        let music_folder_ids = music_folders
            .iter()
            .map(|music_folder| music_folder.id)
            .collect_vec();
        let album_fs_ids = song_paths_to_album_ids(temp_db.pool(), &song_fs_info).await;

        let album_ids = get_artist_and_album_ids(temp_db.pool(), &music_folder_ids, &artist_id)
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

        let (temp_db, _temp_fs, music_folders, song_fs_info) = setup_songs(
            n_folder,
            &vec![n_song; n_folder],
            (0..n_folder * n_song)
                .map(|_| SongTag {
                    artists: vec![artist_name.to_owned()],
                    ..Faker.fake()
                })
                .collect_vec(),
        )
        .await;

        let artist_id = upsert_artists(temp_db.pool(), &[artist_name])
            .await
            .unwrap()
            .remove(0);
        let music_folder_ids = music_folders
            .iter()
            .map(|music_folder| music_folder.id)
            .collect_vec()
            .choose_multiple(&mut rand::thread_rng(), 1)
            .cloned()
            .collect_vec();
        let album_fs_ids = song_paths_to_album_ids(
            temp_db.pool(),
            &song_fs_info
                .into_iter()
                .filter(|(k, _)| music_folder_ids.contains(&k.0))
                .collect(),
        )
        .await;

        let album_ids = get_artist_and_album_ids(temp_db.pool(), &music_folder_ids, &artist_id)
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

        let (temp_db, _temp_fs, music_folders, _) = setup_songs(
            n_folder,
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

        let artist_id = upsert_artists(temp_db.pool(), &[artist_name])
            .await
            .unwrap()
            .remove(0);
        let music_folder_ids = music_folders
            .iter()
            .map(|music_folder| music_folder.id)
            .collect_vec();

        assert!(matches!(
            get_artist_and_album_ids(
                temp_db.pool(),
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

        let (temp_db, _temp_fs, music_folders, song_fs_info) = setup_songs(
            n_folder,
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

        let music_folder_ids = music_folders
            .iter()
            .map(|music_folder| music_folder.id)
            .collect_vec();
        let album_fs_ids = song_paths_to_album_ids(temp_db.pool(), &song_fs_info).await;

        let basic_albums = get_basic_albums(temp_db.pool(), &music_folder_ids[..2], &album_fs_ids)
            .await
            .unwrap()
            .into_iter()
            .sorted_by(|a, b| a.name.cmp(&b.name))
            .collect_vec();

        assert_eq!(basic_albums[0].song_count as usize, n_song);
        assert_eq!(basic_albums[1].song_count as usize, n_song + n_diff);
    }
}
