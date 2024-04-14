use anyhow::Result;
use axum::extract::State;
use diesel::dsl::sum;
use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, PgSortExpressionMethods, QueryDsl,
};
use diesel_async::RunQueryDsl;
use futures::{stream, StreamExt, TryStreamExt};
use nghe_proc_macros::{add_axum_response, add_common_validate};

use crate::models::*;
use crate::open_subsonic::id3::*;
use crate::{Database, DatabasePool};

add_common_validate!(GetTopSongsParams);
add_axum_response!(GetTopSongsBody);

async fn get_top_songs(
    pool: &DatabasePool,
    artist: String,
    count: Option<u32>,
) -> Result<Vec<SongId3Db>> {
    get_song_id3_db()
        .left_join(songs_album_artists::table)
        .inner_join(
            artists::table.on(artists::name.eq(artist).and(
                artists::id
                    .eq(songs_artists::artist_id)
                    .or(artists::id.eq(songs_album_artists::album_artist_id)),
            )),
        )
        .left_join(playbacks::table)
        .order(sum(playbacks::count).desc().nulls_last())
        .limit(count.unwrap_or(50) as _)
        .get_results::<SongId3Db>(&mut pool.get().await?)
        .await
        .map_err(anyhow::Error::from)
}

pub async fn get_top_songs_handler(
    State(database): State<Database>,
    req: GetTopSongsRequest,
) -> GetTopSongsJsonResponse {
    let pool = &database.pool;
    Ok(axum::Json(
        GetTopSongsBody {
            top_songs: TopSongs {
                song: stream::iter(get_top_songs(pool, req.params.artist, req.params.count).await?)
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
    use nghe_types::media_annotation::scrobble::ScrobbleParams;

    use super::*;
    use crate::open_subsonic::media_annotation::test::scrobble;
    use crate::utils::song::test::SongTag;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_get_top_songs_no_empty() {
        let artist_name = "artist";
        let n_song = 20_usize;
        let mut infra = Infra::new().await.n_folder(1).await.add_user(None).await;
        infra.add_songs(
            0,
            (0..n_song)
                .map(|_| SongTag { artists: vec![artist_name.into()], ..Faker.fake() })
                .collect(),
        );
        infra.add_n_song(0, 10).scan(.., None).await;

        let top_songs = get_top_songs(infra.pool(), artist_name.into(), None).await.unwrap();
        assert_eq!(top_songs.len(), n_song);
    }

    #[tokio::test]
    async fn test_get_top_songs_album_artist_no_empty() {
        let artist_name = "artist";
        let n_song = 20_usize;
        let mut infra = Infra::new().await.n_folder(1).await.add_user(None).await;
        infra.add_songs(
            0,
            (0..n_song)
                .map(|_| SongTag { album_artists: vec![artist_name.into()], ..Faker.fake() })
                .collect(),
        );
        infra.add_n_song(0, 10).scan(.., None).await;

        let top_songs = get_top_songs(infra.pool(), artist_name.into(), None).await.unwrap();
        assert_eq!(top_songs.len(), n_song);
    }

    #[tokio::test]
    async fn test_get_top_songs_distinct() {
        let artist_name = "artist";
        let n_song = 20_usize;
        let mut infra =
            Infra::new().await.n_folder(1).await.add_user(None).await.add_user(None).await;
        infra
            .add_songs(
                0,
                (0..n_song)
                    .map(|_| SongTag { artists: vec![artist_name.into()], ..Faker.fake() })
                    .collect(),
            )
            .scan(.., None)
            .await;

        let scrobble_ids = infra.song_ids(..).await[..2].to_vec();
        for _ in 0..5 {
            scrobble(
                infra.pool(),
                infra.user_id(0),
                &ScrobbleParams { ids: scrobble_ids.clone(), times: None, submission: None },
            )
            .await
            .unwrap();
            scrobble(
                infra.pool(),
                infra.user_id(1),
                &ScrobbleParams { ids: scrobble_ids.clone(), times: None, submission: None },
            )
            .await
            .unwrap();
        }

        let top_songs = get_top_songs(infra.pool(), artist_name.into(), None).await.unwrap();
        assert_eq!(top_songs.len(), n_song);
    }
}
