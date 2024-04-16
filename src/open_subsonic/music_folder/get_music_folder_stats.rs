use anyhow::Result;
use axum::extract::State;
use diesel::dsl::{count, count_distinct, sum};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl,
    Queryable, SelectableHelper,
};
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_axum_response, add_common_validate, add_convert_types};

use crate::models::*;
use crate::open_subsonic::sql::coalesce;
use crate::{Database, DatabasePool};

add_common_validate!(GetMusicFolderStatsParams, admin);
add_axum_response!(GetMusicFolderStatsBody);

#[add_convert_types(into = nghe_types::music_folder::get_music_folder_stats::MusicFolderStat)]
#[derive(Debug, Queryable)]
#[diesel(table_name = music_folders)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[cfg_attr(test, derive(PartialEq, Eq))]
struct MusicFolderStat {
    #[diesel(embed)]
    music_folder: music_folders::MusicFolder,
    artist_count: i64,
    album_count: i64,
    song_count: i64,
    user_count: i64,
    total_size: i64,
}

async fn get_music_folder_stats(pool: &DatabasePool) -> Result<Vec<MusicFolderStat>> {
    music_folders::table
        .select((
            music_folders::MusicFolder::as_select(),
            artists::table
                .left_join(songs_album_artists::table)
                .left_join(songs_artists::table)
                .inner_join(
                    songs::table.on(songs::id
                        .eq(songs_album_artists::song_id)
                        .or(songs::id.eq(songs_artists::song_id))),
                )
                .filter(songs::music_folder_id.eq(music_folders::id))
                .select(count_distinct(artists::id))
                .single_value()
                .assume_not_null(),
            songs::table
                .filter(songs::music_folder_id.eq(music_folders::id))
                .select(count_distinct(songs::album_id))
                .single_value()
                .assume_not_null(),
            songs::table
                .filter(songs::music_folder_id.eq(music_folders::id))
                .select(count(songs::id))
                .single_value()
                .assume_not_null(),
            user_music_folder_permissions::table
                .filter(user_music_folder_permissions::music_folder_id.eq(music_folders::id))
                .filter(user_music_folder_permissions::allow)
                .select(count(user_music_folder_permissions::user_id))
                .single_value()
                .assume_not_null(),
            songs::table
                .filter(songs::music_folder_id.eq(music_folders::id))
                .select(coalesce(sum(songs::file_size), 0))
                .single_value()
                .assume_not_null(),
        ))
        .get_results::<MusicFolderStat>(&mut pool.get().await?)
        .await
        .map_err(anyhow::Error::from)
}

pub async fn get_music_folder_stats_handler(
    State(database): State<Database>,
    _: GetMusicFolderStatsRequest,
) -> GetMusicFolderStatsJsonResponse {
    Ok(axum::Json(
        GetMusicFolderStatsBody {
            folder_stats: get_music_folder_stats(&database.pool)
                .await?
                .into_iter()
                .map(MusicFolderStat::into)
                .collect(),
        }
        .into(),
    ))
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};
    use itertools::Itertools;
    use rand::prelude::SliceRandom;

    use super::*;
    use crate::utils::song::test::SongTag;
    use crate::utils::test::Infra;

    #[tokio::test(timeout = "5m")]
    async fn test_get_folder_stats() {
        let n_folder = 10_usize;
        let artists = fake::vec![String; 10..20];
        let album = fake::vec![String; 30..40];

        let mut infra =
            Infra::new().await.n_folder(n_folder).await.add_user(None).await.add_user(None).await;

        let folder_stats = get_music_folder_stats(infra.pool())
            .await
            .unwrap()
            .into_iter()
            .sorted_by_key(|s| s.music_folder.id)
            .collect_vec();
        let folder_fs_stats = infra
            .music_folders
            .iter()
            .cloned()
            .map(|music_folder| MusicFolderStat {
                music_folder,
                artist_count: 0,
                album_count: 0,
                song_count: 0,
                user_count: 2,
                total_size: 0,
            })
            .sorted_by_key(|s| s.music_folder.id)
            .collect_vec();
        assert_eq!(folder_stats, folder_fs_stats);

        infra.permissions(.., ..5, false).await;

        (0..n_folder).for_each(|i| {
            if i < n_folder - 1 {
                let n_song = (10..20).fake();
                infra.add_songs(
                    i,
                    (0..n_song)
                        .map(|_| SongTag {
                            artists: artists
                                .choose_multiple(&mut rand::thread_rng(), (1..2).fake())
                                .cloned()
                                .map(String::into)
                                .collect(),
                            album_artists: artists
                                .choose_multiple(&mut rand::thread_rng(), (1..2).fake())
                                .cloned()
                                .map(String::into)
                                .collect(),
                            album: album
                                .choose(&mut rand::thread_rng())
                                .unwrap()
                                .to_string()
                                .into(),
                            ..Faker.fake()
                        })
                        .collect(),
                );
            }
        });
        infra.scan(.., None).await;

        let folder_stats = get_music_folder_stats(infra.pool())
            .await
            .unwrap()
            .into_iter()
            .sorted_by_key(|s| s.music_folder.id)
            .collect_vec();
        let folder_fs_stats = infra
            .music_folders
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, music_folder)| {
                if i < n_folder - 1 {
                    let slice = i..=i;

                    let artist_count = infra.artist_no_ids(slice.clone()).len() as _;
                    let album_count = infra.album_no_ids(slice.clone()).len() as _;

                    let song_fs_infos = infra.song_fs_infos(slice.clone());
                    let song_count = song_fs_infos.len() as _;
                    let total_size =
                        song_fs_infos.iter().fold(0_u32, |aac, s| aac + s.file_size) as _;

                    MusicFolderStat {
                        music_folder,
                        artist_count,
                        album_count,
                        song_count,
                        user_count: if i < 5 { 0 } else { 2 },
                        total_size,
                    }
                } else {
                    MusicFolderStat {
                        music_folder,
                        artist_count: 0,
                        album_count: 0,
                        song_count: 0,
                        user_count: 2,
                        total_size: 0,
                    }
                }
            })
            .sorted_by_key(|s| s.music_folder.id)
            .collect_vec();
        assert_eq!(folder_stats, folder_fs_stats);
    }
}
