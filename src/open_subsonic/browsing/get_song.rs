use anyhow::Result;
use axum::extract::State;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_validate, wrap_subsonic_response};
use uuid::Uuid;

use crate::models::*;
use crate::open_subsonic::common::id3::db::*;
use crate::open_subsonic::common::id3::response::*;
use crate::open_subsonic::common::music_folder::check_user_music_folder_ids;
use crate::{Database, DatabasePool, OSError};

#[add_validate]
#[derive(Debug)]
pub struct GetSongParams {
    id: Uuid,
}

#[wrap_subsonic_response]
pub struct GetSongBody {
    song: SongId3,
}

async fn get_song(
    pool: &DatabasePool,
    music_folder_ids: &[Uuid],
    song_id: &Uuid,
) -> Result<SongId3Db> {
    songs::table
        .inner_join(songs_artists::table)
        .filter(songs::music_folder_id.eq_any(music_folder_ids))
        .filter(songs::id.eq(song_id))
        .group_by(songs::id)
        .select(SongId3Db::as_select())
        .first::<SongId3Db>(&mut pool.get().await?)
        .await
        .optional()?
        .ok_or_else(|| OSError::NotFound("Song".into()).into())
}

pub async fn get_song_handler(
    State(database): State<Database>,
    req: GetSongRequest,
) -> GetSongJsonResponse {
    let music_folder_ids = check_user_music_folder_ids(&database.pool, &req.user_id, None).await?;

    GetSongBody {
        song: get_song(&database.pool, &music_folder_ids, &req.params.id)
            .await?
            .into_res(&database.pool)
            .await?,
    }
    .into()
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};
    use itertools::Itertools;

    use super::*;
    use crate::utils::song::test::SongTag;
    use crate::utils::test::media::song_paths_to_ids;
    use crate::utils::test::setup::TestInfra;

    async fn get_artist_ids(
        pool: &DatabasePool,
        music_folder_ids: &[Uuid],
        song_id: &Uuid,
    ) -> Vec<Uuid> {
        songs::table
            .inner_join(songs_artists::table)
            .filter(songs::music_folder_id.eq_any(music_folder_ids))
            .filter(songs::id.eq(song_id))
            .select(songs_artists::artist_id)
            .distinct()
            .get_results::<Uuid>(&mut pool.get().await.unwrap())
            .await
            .unwrap()
            .into_iter()
            .sorted()
            .collect()
    }

    #[tokio::test]
    async fn test_get_song_id3() {
        let song_tag = Faker.fake::<SongTag>();

        let (test_infra, song_fs_infos) =
            TestInfra::setup_songs(&[1], vec![song_tag.clone()]).await;

        let music_folder_ids = test_infra.music_folder_ids(..);
        let song_id = song_paths_to_ids(test_infra.pool(), &song_fs_infos).await.remove(0);

        let song_id3 = get_song(test_infra.pool(), &music_folder_ids, &song_id).await.unwrap();
        let artist_ids = get_artist_ids(test_infra.pool(), &music_folder_ids, &song_id).await;

        assert_eq!(song_id3.basic.title, song_tag.title);
        assert_eq!(song_id3.artist_ids.into_iter().sorted().collect_vec(), artist_ids);
    }

    #[tokio::test]
    async fn test_get_song_id3_deny_music_folders() {
        let (test_infra, song_fs_infos) = TestInfra::setup_songs(&[1, 1], None).await;

        let music_folder_ids = test_infra.music_folder_ids(0..=0);
        let song_id = song_paths_to_ids(test_infra.pool(), &song_fs_infos[1..]).await.remove(0);

        assert!(matches!(
            get_song(test_infra.pool(), &music_folder_ids, &song_id)
                .await
                .unwrap_err()
                .root_cause()
                .downcast_ref::<OSError>()
                .unwrap(),
            OSError::NotFound(_)
        ));
    }
}
