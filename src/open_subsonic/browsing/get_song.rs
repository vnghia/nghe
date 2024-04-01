use anyhow::Result;
use axum::extract::State;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_validate, wrap_subsonic_response};
use uuid::Uuid;

use crate::models::*;
use crate::open_subsonic::common::id3::db::*;
use crate::open_subsonic::common::id3::query::*;
use crate::open_subsonic::common::id3::response::*;
use crate::open_subsonic::permission::with_permission;
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

async fn get_song(pool: &DatabasePool, user_id: Uuid, song_id: Uuid) -> Result<SongId3Db> {
    get_song_id3_db()
        .filter(with_permission(user_id))
        .filter(songs::id.eq(song_id))
        .first::<SongId3Db>(&mut pool.get().await?)
        .await
        .optional()?
        .ok_or_else(|| OSError::NotFound("Song".into()).into())
}

pub async fn get_song_handler(
    State(database): State<Database>,
    req: GetSongRequest,
) -> GetSongJsonResponse {
    GetSongBody {
        song: get_song(&database.pool, req.user_id, req.params.id)
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
    use crate::utils::test::Infra;

    async fn get_artist_ids(pool: &DatabasePool, user_id: Uuid, song_id: Uuid) -> Vec<Uuid> {
        // inner join = left join + is not null
        get_basic_artist_id3_db()
            .filter(with_permission(user_id))
            .filter(songs::id.eq(song_id))
            .filter(songs_artists::artist_id.is_not_null())
            .select(artists::id)
            .get_results::<Uuid>(&mut pool.get().await.unwrap())
            .await
            .unwrap()
            .into_iter()
            .unique()
            .sorted()
            .collect()
    }

    #[tokio::test]
    async fn test_get_song_id3() {
        let song_tag = Faker.fake::<SongTag>();
        let mut infra = Infra::new().await.n_folder(1).await.add_user(None).await;
        infra.add_songs(0, vec![song_tag.clone()]).scan(.., None).await;

        let song_id = infra.song_ids(..).await.remove(0);
        let song_id3 = get_song(infra.pool(), infra.user_id(0), song_id).await.unwrap();
        let artist_ids = get_artist_ids(infra.pool(), infra.user_id(0), song_id).await;

        assert_eq!(song_id3.basic.title, song_tag.title);
        assert_eq!(song_id3.artist_ids.into_iter().sorted().collect_vec(), artist_ids);
    }

    #[tokio::test]
    async fn test_get_song_id3_deny_music_folders() {
        let mut infra = Infra::new().await.n_folder(2).await.add_user(None).await;
        infra.add_n_song(0, 1).add_n_song(1, 1).scan(.., None).await;
        infra.only_permissions(.., ..1, true).await;

        let song_id = infra.song_ids(1..).await.remove(0);
        assert!(matches!(
            get_song(infra.pool(), infra.user_id(0), song_id)
                .await
                .unwrap_err()
                .root_cause()
                .downcast_ref::<OSError>()
                .unwrap(),
            OSError::NotFound(_)
        ));
    }
}
