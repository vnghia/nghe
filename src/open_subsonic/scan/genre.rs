use anyhow::Result;
use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
use itertools::Itertools;
use uuid::Uuid;

use crate::models::*;
use crate::DatabasePool;

pub async fn upsert_genres(pool: &DatabasePool, genres: &[genres::Genre]) -> Result<Vec<Uuid>> {
    diesel::insert_into(genres::table)
        .values(genres)
        .on_conflict(genres::value)
        .do_update()
        .set(genres::upserted_at.eq(time::OffsetDateTime::now_utc()))
        .returning(genres::id)
        .get_results::<Uuid>(&mut pool.get().await?)
        .await
        .map_err(anyhow::Error::from)
}

pub async fn upsert_song_genres(
    pool: &DatabasePool,
    song_id: Uuid,
    genre_ids: &[Uuid],
) -> Result<()> {
    diesel::insert_into(songs_genres::table)
        .values(
            genre_ids
                .iter()
                .copied()
                .map(|genre_id| songs_genres::NewSongGenre { song_id, genre_id })
                .collect_vec(),
        )
        .on_conflict((songs_genres::song_id, songs_genres::genre_id))
        .do_update()
        .set(songs_genres::upserted_at.eq(time::OffsetDateTime::now_utc()))
        .execute(&mut pool.get().await?)
        .await?;
    Ok(())
}
