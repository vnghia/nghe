use super::{artist::build_artist_indices, scan_full::scan_full};
use crate::{
    config::{parsing::ParsingConfig, ArtistIndexConfig},
    models::*,
    DatabasePool,
};

use anyhow::Result;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use std::borrow::Cow;
use time::OffsetDateTime;

#[derive(Debug, PartialEq)]
pub enum ScanMode {
    Full,
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
    scan_started_at: &OffsetDateTime,
    scanned_count_or_err: Result<usize>,
) -> Result<()> {
    let (scanned_count, error_message) = match scanned_count_or_err {
        Ok(scanned_count) => (scanned_count, None),
        Err(e) => {
            tracing::error!("error while scanning: {:?}", e);
            (0, Some::<Cow<'_, str>>(e.to_string().into()))
        }
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
    artist_index_config: &ArtistIndexConfig,
    music_folders: &[music_folders::MusicFolder],
    parsing_config: &ParsingConfig,
) -> Result<()> {
    let scan_started_at = start_scan(pool).await?;

    let scanned_count_or_err = match scan_mode {
        ScanMode::Full => scan_full(pool, &scan_started_at, music_folders, parsing_config)
            .await
            .map(|result| result.0),
    };
    build_artist_indices(pool, artist_index_config).await?;
    finish_scan(pool, &scan_started_at, scanned_count_or_err).await?;

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
