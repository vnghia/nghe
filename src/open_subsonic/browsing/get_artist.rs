use crate::{
    models::*, open_subsonic::common::id3::ArtistId3, DatabasePool, OSResult, OpenSubsonicError,
};

use diesel::{
    dsl::{count_distinct, sql},
    sql_types, BoolExpressionMethods, ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl,
};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

pub async fn get_artist_and_album_ids(
    pool: &DatabasePool,
    music_folder_ids: &[Uuid],
    artist_id: &Uuid,
) -> OSResult<(ArtistId3, Vec<Uuid>)> {
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
        .ok_or(OpenSubsonicError::NotFound {
            message: Some("artist not found".into()),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        open_subsonic::scan::artist::upsert_artists,
        utils::{
            song::tag::SongTag,
            test::{media::song_paths_to_album_ids, setup::setup_users_and_songs},
        },
    };

    use fake::{Fake, Faker};
    use itertools::Itertools;
    use rand::seq::SliceRandom;

    #[tokio::test]
    async fn test_get_artist_own_albums() {
        let artist_name = "artist";
        let n_song = 10_usize;

        let (temp_db, _, _temp_fs, music_folders, song_fs_info) = setup_users_and_songs(
            0,
            1,
            &[],
            &[n_song],
            (0..n_song)
                .map(|i| SongTag {
                    album_artists: if i < 5 {
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
        let album_fs_ids = song_paths_to_album_ids(
            temp_db.pool(),
            &song_fs_info
                .into_iter()
                .filter(|(_, v)| v.album_artists.contains(&artist_name.to_owned()))
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
    async fn test_get_artist_featured_in_albums() {
        let artist_name = "artist";
        let n_song = 10_usize;

        let (temp_db, _, _temp_fs, music_folders, song_fs_info) = setup_users_and_songs(
            0,
            1,
            &[],
            &[n_song],
            (0..n_song)
                .map(|i| SongTag {
                    artists: if i < 5 {
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
        let album_fs_ids = song_paths_to_album_ids(
            temp_db.pool(),
            &song_fs_info
                .into_iter()
                .filter(|(_, v)| v.artists.contains(&artist_name.to_owned()))
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
    async fn test_get_artist_distinct_albums() {
        let artist_name = "artist";
        let album_names = ["album1", "album2"];
        let n_song = 10_usize;

        let (temp_db, _, _temp_fs, music_folders, song_fs_info) = setup_users_and_songs(
            0,
            1,
            &[],
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
        let album_fs_ids = song_paths_to_album_ids(
            temp_db.pool(),
            &song_fs_info
                .into_iter()
                .filter(|(_, v)| v.artists.contains(&artist_name.to_owned()))
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

        assert_eq!(album_ids.len(), album_names.len());
        assert_eq!(album_ids, album_fs_ids);
    }

    #[tokio::test]
    async fn test_get_artist_albums_multiple_music_folders() {
        let artist_name = "artist";
        let n_folder = 5_usize;
        let n_song = 10_usize;

        let (temp_db, _, _temp_fs, music_folders, song_fs_info) = setup_users_and_songs(
            0,
            n_folder,
            &[],
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
            .choose_multiple(&mut rand::thread_rng(), 2)
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
        let n_folder = 5_usize;
        let n_scan_folder = 2_usize;
        let n_song = 10_usize;

        let (temp_db, _, _temp_fs, music_folders, _) = setup_users_and_songs(
            0,
            n_folder,
            &[],
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
            .await,
            Err(OpenSubsonicError::NotFound { message: _ })
        ));
    }
}
