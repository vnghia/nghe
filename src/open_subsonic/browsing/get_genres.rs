use anyhow::Result;
use axum::extract::State;
use diesel::QueryDsl;
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_axum_response, add_common_validate};
use uuid::Uuid;

use crate::open_subsonic::id3::*;
use crate::open_subsonic::permission::with_permission;
use crate::{Database, DatabasePool};

add_common_validate!(GetGenresParams);
add_axum_response!(GenresBody);

async fn get_genres(pool: &DatabasePool, user_id: Uuid) -> Result<Vec<GenreId3Db>> {
    get_genre_id3_db()
        .filter(with_permission(user_id))
        .get_results::<GenreId3Db>(&mut pool.get().await?)
        .await
        .map_err(anyhow::Error::from)
}

pub async fn get_genres_handler(
    State(database): State<Database>,
    req: GetGenresRequest,
) -> GenresJsonResponse {
    Ok(axum::Json(
        GenresBody {
            genres: Genres {
                genre: get_genres(&database.pool, req.user_id)
                    .await?
                    .into_iter()
                    .map(GenreId3Db::into)
                    .collect(),
            },
        }
        .into(),
    ))
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};
    use itertools::Itertools;

    use super::*;
    use crate::models::*;
    use crate::utils::song::test::SongTag;
    use crate::utils::test::Infra;

    async fn get_genre_values(pool: &DatabasePool, user_id: Uuid) -> Vec<String> {
        get_genres(pool, user_id)
            .await
            .unwrap()
            .into_iter()
            .map(|v| v.value.value.into())
            .sorted()
            .collect()
    }

    #[tokio::test]
    async fn test_get_genres() {
        let genre_values = ["genre1", "genre2"];
        let n_song = 10_usize;
        let mut infra = Infra::new().await.n_folder(1).await.add_user(None).await;
        infra
            .add_songs(
                0,
                (0..n_song)
                    .map(|_| SongTag {
                        genres: genre_values.into_iter().map(genres::Genre::from).collect(),
                        ..Faker.fake()
                    })
                    .collect(),
            )
            .scan(.., None)
            .await;

        let genre_db_values = get_genre_values(infra.pool(), infra.user_id(0)).await;
        assert_eq!(genre_db_values, genre_values);
    }

    #[tokio::test]
    async fn test_get_genres_partial() {
        let genre_values = ["genre1", "genre2"];
        let n_song = 10_usize;
        let mut infra = Infra::new().await.n_folder(2).await.add_user(None).await;
        genre_values.into_iter().enumerate().for_each(|(i, v)| {
            infra.add_songs(
                i,
                (0..n_song).map(|_| SongTag { genres: vec![v.into()], ..Faker.fake() }).collect(),
            );
        });
        infra.scan(.., None).await;
        infra.permissions(.., 1.., false).await;

        let genre_db_values = get_genre_values(infra.pool(), infra.user_id(0)).await;
        assert_eq!(genre_db_values, vec!["genre1".to_string()]);
    }

    #[tokio::test]
    async fn test_get_genres_count() {
        let genre_value = "genre1";
        let n_folder = 2_usize;
        let n_song = 10_usize;
        let mut infra = Infra::new().await.n_folder(n_folder).await.add_user(None).await;
        (0..n_folder).for_each(|i| {
            infra.add_songs(
                i,
                (0..n_song)
                    .map(|_| SongTag { genres: vec![genre_value.into()], ..Faker.fake() })
                    .collect(),
            );
        });
        infra.scan(.., None).await;

        let genre_song_count =
            get_genres(infra.pool(), infra.user_id(0)).await.unwrap()[0].song_count;
        assert_eq!(genre_song_count as usize, 2 * n_song);

        infra.permissions(.., 1.., false).await;
        let genre_song_count =
            get_genres(infra.pool(), infra.user_id(0)).await.unwrap()[0].song_count;
        assert_eq!(genre_song_count as usize, n_song);
    }
}
