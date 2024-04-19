use anyhow::Result;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use super::*;
use crate::models::*;
use crate::DatabasePool;

pub async fn get_songs(pool: &DatabasePool, song_ids: &[Uuid]) -> Result<Vec<SongId3Db>> {
    get_song_id3_db()
        .filter(songs::id.eq_any(song_ids))
        .get_results::<SongId3Db>(&mut pool.get().await?)
        .await
        .map_err(anyhow::Error::from)
}
