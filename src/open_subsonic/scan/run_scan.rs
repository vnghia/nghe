use std::borrow::Cow;

use anyhow::Result;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use time::OffsetDateTime;

use super::artist::build_artist_indices;
use super::scan_full::scan_full;
use crate::config::parsing::ParsingConfig;
use crate::config::{ArtistIndexConfig, ScanConfig};
use crate::models::*;
use crate::DatabasePool;

#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ScanMode {
    Full,
}

#[derive(Debug)]
#[cfg_attr(test, derive(Clone))]
pub struct ScanStatistic {
    pub scanned_song_count: usize,
    pub upserted_song_count: usize,
    pub deleted_song_count: usize,
    pub deleted_album_count: usize,
    pub deleted_artist_count: usize,
    pub scan_error_count: usize,
}

pub async fn start_scan(pool: &DatabasePool) -> Result<OffsetDateTime> {
    diesel::insert_into(scans::table)
        .default_values()
        .returning(scans::started_at)
        .get_result::<OffsetDateTime>(&mut pool.get().await?)
        .await
        .map_err(anyhow::Error::from)
}

pub async fn finish_scan(
    pool: &DatabasePool,
    scan_started_at: OffsetDateTime,
    scan_result: Result<&ScanStatistic, &anyhow::Error>,
) -> Result<()> {
    let (scanned_count, error_message) = match scan_result {
        Ok(r) => (r.scanned_song_count, None),
        Err(e) => (0, Some::<Cow<'_, str>>(e.to_string().into())),
    };
    diesel::update(scans::table.filter(scans::started_at.eq(scan_started_at)))
        .set(&scans::FinishScan {
            is_scanning: false,
            finished_at: OffsetDateTime::now_utc(),
            scanned_count: scanned_count as i64,
            error_message,
        })
        .execute(&mut pool.get().await?)
        .await?;
    Ok(())
}

pub async fn run_scan(
    pool: &DatabasePool,
    scan_mode: ScanMode,
    music_folders: &[music_folders::MusicFolder],
    artist_index_config: &ArtistIndexConfig,
    parsing_config: &ParsingConfig,
    scan_config: &ScanConfig,
) -> Result<()> {
    let scan_started_at = start_scan(pool).await?;

    let scan_result = match scan_mode {
        ScanMode::Full => {
            scan_full(pool, scan_started_at, music_folders, parsing_config, scan_config).await
        }
    };
    build_artist_indices(pool, artist_index_config).await?;
    finish_scan(pool, scan_started_at, scan_result.as_ref()).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test::TemporaryDatabase;

    #[tokio::test]
    async fn test_start_scan_twice() {
        let temp_db = TemporaryDatabase::new_from_env().await;
        start_scan(temp_db.pool()).await.unwrap();
        assert!(start_scan(temp_db.pool()).await.is_err());
    }
}
