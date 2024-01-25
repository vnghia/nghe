use crate::models::*;
use crate::{DatabasePool, OSResult};

use diesel_async::RunQueryDsl;
use itertools::Itertools;
use uuid::Uuid;

pub async fn upsert_song_artists(
    pool: &DatabasePool,
    song_id: Uuid,
    artist_ids: &[Uuid],
) -> OSResult<()> {
    diesel::insert_into(songs_artists::table)
        .values(
            artist_ids
                .iter()
                .cloned()
                .map(|artist_id| songs_artists::NewSongArtist { song_id, artist_id })
                .collect_vec(),
        )
        .on_conflict_do_nothing()
        .execute(&mut pool.get().await?)
        .await?;
    Ok(())
}
