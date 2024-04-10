use std::ops::Add;

use anyhow::Result;
use axum::extract::State;
use diesel::upsert::excluded;
use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
use itertools::Itertools;
use nghe_proc_macros::{add_axum_response, add_common_validate};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::models::*;
use crate::{Database, DatabasePool, OSError};

add_common_validate!(ScrobbleParams);
add_axum_response!(ScrobbleBody);

async fn scrobble(pool: &DatabasePool, user_id: Uuid, params: &ScrobbleParams) -> Result<()> {
    if params.submission {
        if let Some(ref times) = params.times {
            if params.ids.len() != times.len() {
                anyhow::bail!(OSError::InvalidParameter(
                    "song ids and times must have the same size".into()
                ))
            } else {
                // convert milliseconds to nanoseconds
                let updated_ats: Vec<_> = times
                    .iter()
                    .map(|t| OffsetDateTime::from_unix_timestamp_nanos(t * 1000000))
                    .try_collect()?;
                diesel::insert_into(playbacks::table)
                    .values(
                        params
                            .ids
                            .iter()
                            .copied()
                            .zip(updated_ats)
                            .map(|(song_id, updated_at)| playbacks::NewScrobble {
                                user_id,
                                song_id,
                                updated_at: Some(updated_at),
                            })
                            .collect_vec(),
                    )
                    .on_conflict((playbacks::user_id, playbacks::song_id))
                    .do_update()
                    .set((
                        playbacks::count.eq(playbacks::count.add(1)),
                        playbacks::updated_at.eq(excluded(playbacks::updated_at)),
                    ))
                    .execute(&mut pool.get().await?)
                    .await?;
            }
        } else {
            diesel::insert_into(playbacks::table)
                .values(
                    params
                        .ids
                        .iter()
                        .copied()
                        .map(|song_id| playbacks::NewScrobble {
                            user_id,
                            song_id,
                            updated_at: None,
                        })
                        .collect_vec(),
                )
                .on_conflict((playbacks::user_id, playbacks::song_id))
                .do_update()
                .set(playbacks::count.eq(playbacks::count.add(1)))
                .execute(&mut pool.get().await?)
                .await?;
        }
    }
    Ok(())
}

pub async fn scrobble_handler(
    State(database): State<Database>,
    req: ScrobbleRequest,
) -> ScrobbleJsonResponse {
    scrobble(&database.pool, req.user_id, &req.params).await?;
    Ok(axum::Json(ScrobbleBody {}.into()))
}

#[cfg(test)]
mod tests {
    use diesel::QueryDsl;
    use fake::faker::time::en::*;
    use fake::Fake;
    use time::macros::datetime;

    use super::*;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_scrobble() {
        let mut infra = Infra::new().await.n_folder(1).await.add_user(None).await;
        infra.add_n_song(0, 1).scan(.., None).await;
        let user_id = infra.user_id(0);
        let song_id = infra.song_ids(..).await[0];

        for _ in 0..50 {
            scrobble(
                infra.pool(),
                user_id,
                &ScrobbleParams { ids: vec![song_id], times: None, submission: true },
            )
            .await
            .unwrap();
        }
        let play_count = playbacks::table
            .filter(playbacks::user_id.eq(user_id))
            .filter(playbacks::song_id.eq(song_id))
            .select(playbacks::count)
            .get_result::<i32>(&mut infra.pool().get().await.unwrap())
            .await
            .unwrap();
        assert_eq!(50, play_count);
    }

    #[tokio::test]
    async fn test_scrobble_multiples() {
        let n_song = 10_usize;
        let more_play_count = 10_usize;
        let mut infra = Infra::new().await.n_folder(1).await.add_user(None).await;
        infra.add_n_song(0, n_song).scan(.., None).await;
        let user_id = infra.user_id(0);
        let song_ids = infra.song_ids(..).await;
        let mut play_counts = vec![0; n_song];

        for (i, song_id) in song_ids.iter().copied().enumerate() {
            let play_count = (5..10).fake();
            for _ in 0..play_count {
                scrobble(
                    infra.pool(),
                    user_id,
                    &ScrobbleParams { ids: vec![song_id], times: None, submission: true },
                )
                .await
                .unwrap();
            }
            play_counts[i] = play_count + more_play_count;
        }

        for _ in 0..more_play_count {
            scrobble(
                infra.pool(),
                user_id,
                &ScrobbleParams { ids: song_ids.clone(), times: None, submission: true },
            )
            .await
            .unwrap();
        }

        for (i, song_id) in song_ids.iter().copied().enumerate() {
            let play_count = playbacks::table
                .filter(playbacks::user_id.eq(user_id))
                .filter(playbacks::song_id.eq(song_id))
                .select(playbacks::count)
                .get_result::<i32>(&mut infra.pool().get().await.unwrap())
                .await
                .unwrap();
            assert_eq!(play_counts[i], play_count as usize);
        }
    }

    #[tokio::test]
    async fn test_scrobble_multiples_time() {
        let n_song = 10_usize;
        let mut infra = Infra::new().await.n_folder(1).await.add_user(None).await;
        infra.add_n_song(0, n_song).scan(.., None).await;
        let user_id = infra.user_id(0);
        let song_ids = infra.song_ids(..).await;
        let start_dt = datetime!(1000-01-01 0:00 UTC);
        let end_dt = datetime!(2000-01-01 0:00 UTC);
        let times = (0..n_song).map(|_| DateTimeBetween(start_dt, end_dt).fake()).collect_vec();

        for song_id in song_ids.iter().copied() {
            for _ in 0..10 {
                scrobble(
                    infra.pool(),
                    user_id,
                    &ScrobbleParams { ids: vec![song_id], times: None, submission: true },
                )
                .await
                .unwrap();
            }
        }
        scrobble(
            infra.pool(),
            user_id,
            &ScrobbleParams {
                ids: song_ids.clone(),
                times: Some(
                    times
                        .iter()
                        .map(|t: &OffsetDateTime| t.unix_timestamp_nanos() / 1000000)
                        .collect(),
                ),
                submission: true,
            },
        )
        .await
        .unwrap();

        for (i, song_id) in song_ids.iter().copied().enumerate() {
            let (play_count, time) = playbacks::table
                .filter(playbacks::user_id.eq(user_id))
                .filter(playbacks::song_id.eq(song_id))
                .select((playbacks::count, playbacks::updated_at))
                .get_result::<(i32, OffsetDateTime)>(&mut infra.pool().get().await.unwrap())
                .await
                .unwrap();
            assert_eq!(11, play_count);
            assert_eq!(times[i], time);
        }
    }
}
