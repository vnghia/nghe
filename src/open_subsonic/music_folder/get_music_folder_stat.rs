use anyhow::Result;
use axum::extract::State;
use diesel::dsl::{count, count_distinct, sum};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl,
    SelectableHelper,
};
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_axum_response, add_common_validate};
use uuid::Uuid;

use crate::models::*;
use crate::open_subsonic::sql::coalesce;
use crate::{Database, DatabasePool};

add_common_validate!(GetMusicFolderStatParams, admin);
add_axum_response!(GetMusicFolderStatBody);

async fn get_music_folder_stat(
    pool: &DatabasePool,
    id: Uuid,
) -> Result<music_folders::MusicFolderStat> {
    music_folders::table
        .filter(music_folders::id.eq(id))
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
                .filter(songs::music_folder_id.eq(id))
                .select(count_distinct(artists::id))
                .single_value()
                .assume_not_null(),
            songs::table
                .filter(songs::music_folder_id.eq(id))
                .select(count_distinct(songs::album_id))
                .single_value()
                .assume_not_null(),
            songs::table
                .filter(songs::music_folder_id.eq(id))
                .select(count(songs::id))
                .single_value()
                .assume_not_null(),
            user_music_folder_permissions::table
                .filter(user_music_folder_permissions::music_folder_id.eq(id))
                .select(count(user_music_folder_permissions::user_id))
                .single_value()
                .assume_not_null(),
            songs::table
                .filter(songs::music_folder_id.eq(id))
                .select(coalesce(sum(songs::file_size), 0))
                .single_value()
                .assume_not_null(),
        ))
        .get_result::<music_folders::MusicFolderStat>(&mut pool.get().await?)
        .await
        .map_err(anyhow::Error::from)
}

pub async fn get_music_folder_stat_handler(
    State(database): State<Database>,
    req: GetMusicFolderStatRequest,
) -> GetMusicFolderStatJsonResponse {
    Ok(axum::Json(
        GetMusicFolderStatBody {
            stat: get_music_folder_stat(&database.pool, req.params.id).await?.into(),
        }
        .into(),
    ))
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};
    use rand::prelude::SliceRandom;

    use super::*;
    use crate::utils::song::test::SongTag;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_get_folder_stat_empty() {
        let infra = Infra::new().await.add_user(None).await.add_folder(false).await;
        let stat = get_music_folder_stat(infra.pool(), infra.music_folder_id(0)).await.unwrap();
        assert_eq!(
            music_folders::MusicFolderStat {
                music_folder: infra.music_folders[0].clone(),
                artist_count: 0,
                album_count: 0,
                song_count: 0,
                user_count: 0,
                total_size: 0,
            },
            stat
        );
    }

    #[tokio::test]
    async fn test_get_folder_stat_empty_with_user() {
        let infra = Infra::new().await.add_user(None).await.add_folder(true).await;
        let stat = get_music_folder_stat(infra.pool(), infra.music_folder_id(0)).await.unwrap();
        assert_eq!(
            music_folders::MusicFolderStat {
                music_folder: infra.music_folders[0].clone(),
                artist_count: 0,
                album_count: 0,
                song_count: 0,
                user_count: 1,
                total_size: 0,
            },
            stat
        );
    }

    #[tokio::test]
    async fn test_get_folder_stat() {
        let n_song = (10..20).fake();
        let artists = fake::vec![String; 2..5];
        let album = fake::vec![String; 2..5];

        let mut thread_rng = rand::thread_rng();

        let mut infra = Infra::new().await.add_user(None).await.add_folder(true).await;
        infra
            .add_songs(
                0,
                (0..n_song)
                    .map(|_| SongTag {
                        artists: artists
                            .choose_multiple(&mut thread_rng, (1..2).fake())
                            .cloned()
                            .map(String::into)
                            .collect(),
                        album_artists: artists
                            .choose_multiple(&mut thread_rng, (1..2).fake())
                            .cloned()
                            .map(String::into)
                            .collect(),
                        album: album.choose(&mut thread_rng).unwrap().to_string().into(),
                        ..Faker.fake()
                    })
                    .collect(),
            )
            .scan(.., None)
            .await;

        let artist_count = infra.artist_no_ids(..).len() as _;
        let album_count = infra.album_no_ids(..).len() as _;

        let song_fs_infos = infra.song_fs_infos(..);
        let song_count = song_fs_infos.len() as _;
        let total_size = song_fs_infos.iter().fold(0_u32, |aac, s| aac + s.file_size) as _;

        let stat = get_music_folder_stat(infra.pool(), infra.music_folder_id(0)).await.unwrap();
        assert_eq!(
            music_folders::MusicFolderStat {
                music_folder: infra.music_folders[0].clone(),
                artist_count,
                album_count,
                song_count,
                user_count: 1,
                total_size,
            },
            stat
        );
    }
}
