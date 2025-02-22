use itertools::{EitherOrBoth, Itertools};
pub use nghe_api::media_annotation::scrobble::{Request, Response};
use nghe_proc_macro::handler;
use uuid::Uuid;

use crate::database::Database;
use crate::orm::playbacks;
use crate::{Error, error};

#[handler]
pub async fn handler(
    database: &Database,
    user_id: Uuid,
    request: Request,
) -> Result<Response, Error> {
    let submission = request.submission.unwrap_or(true);
    if submission {
        let values: Vec<_> = request
            .ids
            .into_iter()
            .zip_longest(request.times.unwrap_or_default())
            .map(|data| match data {
                EitherOrBoth::Both(song_id, updated_at) => {
                    Ok(playbacks::Scrobble { user_id, song_id, updated_at })
                }
                EitherOrBoth::Left(song_id) => Ok(playbacks::Scrobble {
                    user_id,
                    song_id,
                    updated_at: time::OffsetDateTime::now_utc(),
                }),
                EitherOrBoth::Right(_) => error::Kind::InvalidScrobbleTimeSize.into(),
            })
            .try_collect()?;
        playbacks::Scrobble::upsert(database, &values).await?;
    }

    Ok(Response)
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use diesel::{ExpressionMethods, QueryDsl};
    use diesel_async::RunQueryDsl;
    use fake::faker::time::en::*;
    use fake::{Fake, Faker};
    use rstest::rstest;
    use time::OffsetDateTime;
    use time::macros::datetime;

    use super::*;
    use crate::test::{Mock, mock};

    #[rstest]
    #[tokio::test]
    async fn test_scrobble(
        #[future(awt)] mock: Mock,
        #[values(2, 5)] n_song: usize,
        #[values(0, 3)] n_time: usize,
        #[values(10, 20)] n_play: usize,
    ) {
        let user_id = mock.user_id(0).await;

        let mut music_folder = mock.music_folder(0).await;
        music_folder.add_audio().n_song(n_song).call().await;
        let ids: Vec<_> = music_folder.database.keys().copied().collect();

        let start_dt = datetime!(2000-01-01 0:00 UTC);
        let end_dt = datetime!(2020-01-01 0:00 UTC);
        let times = fake::vec![
            OffsetDateTime as DateTimeBetween(start_dt, end_dt);
            n_time
        ];

        for i in 0..n_play {
            let times = if i < n_play - 1 {
                if Faker.fake() {
                    Some(fake::vec![
                        OffsetDateTime as DateTimeBetween(start_dt, end_dt);
                        0..(n_song - (0..2).fake::<usize>())
                    ])
                } else {
                    None
                }
            } else {
                Some(times.clone())
            };

            let result = handler(
                mock.database(),
                user_id,
                Request { ids: ids.clone(), times, submission: None },
            )
            .await;
            assert_eq!(result.is_ok(), i < n_play - 1 || n_song >= n_time);
        }

        for (i, id) in ids.into_iter().enumerate() {
            let (count, time) = playbacks::table
                .filter(playbacks::user_id.eq(user_id))
                .filter(playbacks::song_id.eq(id))
                .select((playbacks::count, playbacks::updated_at))
                .get_result::<(i32, OffsetDateTime)>(&mut mock.get().await)
                .await
                .unwrap();

            let count: usize = count.try_into().unwrap();
            assert_eq!(count, if n_song >= n_time { n_play } else { n_play - 1 });

            if n_song >= n_time {
                if i >= n_time {
                    let now = OffsetDateTime::now_utc();
                    assert!((now - time).as_seconds_f32() < 1.0);
                } else {
                    assert_eq!(time, times[i]);
                }
            }
        }
    }
}
