use anyhow::Result;
use diesel::{ExpressionMethods, OptionalExtension};
use diesel_async::RunQueryDsl;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::models::*;
use crate::utils::song::SongLyric;
use crate::DatabasePool;

impl SongLyric {
    pub async fn upsert_lyric(&self, pool: &DatabasePool, song_id: Uuid) -> Result<()> {
        let lyric_key = self.as_key(song_id);
        let lyric_hash = self.lyric_hash as i64;
        let lyric_size = self.lyric_size as i64;

        if let Some((lyric_db_hash, lyric_db_size)) = diesel::update(&lyric_key)
            .set(lyrics::scanned_at.eq(OffsetDateTime::now_utc()))
            .returning((lyrics::lyric_hash, lyrics::lyric_size))
            .get_result::<(i64, i64)>(&mut pool.get().await?)
            .await
            .optional()?
        {
            if lyric_db_hash == lyric_hash && lyric_db_size == lyric_size {
                Ok(())
            } else {
                let update_lyric = self.as_update();
                diesel::update(&lyric_key)
                    .set(update_lyric)
                    .execute(&mut pool.get().await?)
                    .await?;
                Ok(())
            }
        } else {
            let update_lyric = self.as_update();
            let new_lyric = lyrics::NewLyric { key: lyric_key, update: update_lyric };
            diesel::insert_into(lyrics::table)
                .values(new_lyric)
                .execute(&mut pool.get().await?)
                .await?;
            Ok(())
        }
    }
}
