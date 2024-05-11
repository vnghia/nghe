use anyhow::Result;
use axum::extract::State;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use futures::{stream, StreamExt, TryStreamExt};
use nghe_proc_macros::{
    add_axum_response, add_common_validate, add_count_offset, add_permission_filter,
};
use uuid::Uuid;

use crate::models::*;
use crate::open_subsonic::id3::*;
use crate::open_subsonic::permission::check_permission;
use crate::{Database, DatabasePool};

add_common_validate!(GetSongsByGenreParams);
add_axum_response!(GetSongsByGenreBody);

async fn get_songs_by_genre(
    pool: &DatabasePool,
    user_id: Uuid,
    params: GetSongsByGenreParams,
) -> Result<Vec<SongId3Db>> {
    let GetSongsByGenreParams { genre, count, offset, music_folder_ids } = params;
    let count = count.unwrap_or(10);
    let offset = offset.unwrap_or(0);

    check_permission(pool, user_id, &music_folder_ids).await?;

    #[add_permission_filter]
    #[add_count_offset]
    get_song_id3_db()
        .order(songs::title.asc())
        .filter(genres::value.eq(genre))
        .get_results(&mut pool.get().await?)
        .await
        .map_err(anyhow::Error::from)
}

pub async fn get_songs_by_genre_handler(
    State(database): State<Database>,
    req: GetSongsByGenreRequest,
) -> GetSongsByGenreJsonResponse {
    let pool = &database.pool;
    Ok(axum::Json(
        GetSongsByGenreBody {
            songs_by_genre: SongsByGenre {
                song: stream::iter(get_songs_by_genre(pool, req.user_id, req.params).await?)
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
    use fake::{Fake, Faker};

    use super::*;
    use crate::utils::song::test::SongTag;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_get_songs_by_genre() {
        let genre_values = ["genre1", "genre2"];
        let n_song = 10_usize;
        let n_diff = 3_usize;
        let mut infra = Infra::new().await.n_folder(1).await.add_user(None).await;
        infra
            .add_songs(
                0,
                (0..n_song + n_diff)
                    .map(|_| SongTag { genres: vec![genre_values[0].into()], ..Faker.fake() })
                    .collect(),
            )
            .await
            .add_songs(
                0,
                (0..n_song - n_diff)
                    .map(|_| SongTag { genres: vec![genre_values[1].into()], ..Faker.fake() })
                    .collect(),
            )
            .await
            .scan(.., None)
            .await;

        let songs = get_songs_by_genre(
            infra.pool(),
            infra.user_id(0),
            GetSongsByGenreParams {
                genre: genre_values[0].to_string(),
                count: Some(20),
                offset: None,
                music_folder_ids: None,
            },
        )
        .await
        .unwrap();
        assert_eq!(songs.len(), n_song + n_diff);
    }
}
