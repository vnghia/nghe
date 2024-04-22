use anyhow::Result;
use axum::extract::State;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use futures::{stream, StreamExt, TryStreamExt};
use nghe_proc_macros::{add_axum_response, add_common_validate, add_permission_filter};
use uuid::Uuid;

use crate::models::*;
use crate::open_subsonic::common::sql::random;
use crate::open_subsonic::id3::*;
use crate::open_subsonic::permission::check_permission;
use crate::{Database, DatabasePool};

add_common_validate!(GetRandomSongsParams);
add_axum_response!(GetRandomSongsBody);

async fn get_random_songs(
    pool: &DatabasePool,
    user_id: Uuid,
    params: GetRandomSongsParams,
) -> Result<Vec<SongId3Db>> {
    let GetRandomSongsParams { count, music_folder_ids, from_year, to_year, genre } = params;

    check_permission(pool, user_id, &music_folder_ids).await?;

    let mut query = {
        #[add_permission_filter]
        get_song_id3_db().order(random()).limit(count.unwrap_or(10) as _).into_boxed()
    };
    if let Some(from_year) = from_year {
        query = query.filter(songs::year.ge(from_year as i16));
    }
    if let Some(to_year) = to_year {
        query = query.filter(songs::year.le(to_year as i16));
    }
    if let Some(genre) = genre {
        query = query.filter(genres::value.eq(genre));
    }
    query.get_results(&mut pool.get().await?).await.map_err(anyhow::Error::from)
}

pub async fn get_random_songs_handler(
    State(database): State<Database>,
    req: GetRandomSongsRequest,
) -> GetRandomSongsJsonResponse {
    let pool = &database.pool;
    Ok(axum::Json(
        GetRandomSongsBody {
            random_songs: RandomSongs {
                song: stream::iter(get_random_songs(pool, req.user_id, req.params).await?)
                    .then(|v| async move { v.into(pool).await })
                    .try_collect()
                    .await?,
            },
        }
        .into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_get_random_songs() {
        let mut infra = Infra::new().await.n_folder(1).await.add_user(None).await;
        infra.add_n_song(0, 20).scan(.., None).await;
        get_random_songs(
            infra.pool(),
            infra.user_id(0),
            GetRandomSongsParams {
                count: None,
                music_folder_ids: None,
                from_year: Some(1000),
                to_year: Some(2000),
                genre: Some("genre".into()),
            },
        )
        .await
        .unwrap();
    }
}
