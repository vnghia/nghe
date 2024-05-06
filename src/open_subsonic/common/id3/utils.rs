use std::collections::HashMap;

use anyhow::Result;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use itertools::Itertools;
use uuid::Uuid;

use super::*;
use crate::models::*;
use crate::DatabasePool;

pub async fn get_songs(pool: &DatabasePool, song_ids: &[Uuid]) -> Result<Vec<SongId3Db>> {
    let songs = get_song_id3_db()
        .filter(songs::id.eq_any(song_ids))
        .get_results(&mut pool.get().await?)
        .await?;
    let song_positions: HashMap<_, _> =
        song_ids.iter().enumerate().map(|(i, id)| (id, i)).collect();
    if songs.len() != song_positions.len() {
        tracing::error!(
            song_id_len = song_ids.len(),
            song_id3_len = songs.len(),
            "song id and song id3 has different length",
        );
    }
    Ok(songs
        .into_iter()
        .sorted_by_key(|s| song_positions.get(&s.basic.id).copied().unwrap_or(usize::MAX))
        .collect())
}

#[cfg(test)]
mod tests {
    use rand::seq::SliceRandom;

    use super::*;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_get_songs_ordered() {
        let mut infra = Infra::new().await.n_folder(1).await;
        infra.add_n_song(0, 10).await.scan(.., None).await;

        let mut song_ids = infra.song_ids(..).await;
        song_ids.shuffle(&mut rand::thread_rng());
        let song_id3_ids = get_songs(infra.pool(), &song_ids)
            .await
            .unwrap()
            .into_iter()
            .map(|s| s.basic.id)
            .collect_vec();
        assert_eq!(song_ids, song_id3_ids);
    }
}
