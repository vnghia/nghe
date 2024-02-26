use super::scan_full::scan_full;
use crate::{models::*, DatabasePool, OSResult};

use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use time::OffsetDateTime;

#[derive(Debug, PartialEq)]
pub enum ScanMode {
    Full,
}

pub async fn start_scan(pool: &DatabasePool) -> OSResult<OffsetDateTime> {
    Ok(diesel::insert_into(scans::table)
        .default_values()
        .returning(scans::started_at)
        .get_result::<OffsetDateTime>(&mut pool.get().await?)
        .await?)
}

pub async fn finish_scan(
    pool: &DatabasePool,
    scan_started_at: &OffsetDateTime,
    scanned_count_or_err: OSResult<usize>,
) -> OSResult<()> {
    let (scanned_count, error_message) = match scanned_count_or_err {
        Ok(scanned_count) => (scanned_count, None),
        Err(e) => (0, Some(e.into_cow_str())),
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
) -> OSResult<()> {
    let scan_started_at = start_scan(pool).await?;

    let scanned_count_or_err = match scan_mode {
        ScanMode::Full => scan_full(pool, &scan_started_at, music_folders)
            .await
            .map(|result| result.0),
    };
    if let Err(e) = &scanned_count_or_err {
        tracing::error!("error while scanning {:?}", e);
    }

    finish_scan(pool, &scan_started_at, scanned_count_or_err).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{utils::test::TemporaryDatabase, OpenSubsonicError};

    #[tokio::test]
    async fn test_start_scan_twice() {
        let temp_db = TemporaryDatabase::new_from_env().await;
        start_scan(temp_db.pool()).await.unwrap();

        assert!(matches!(
            start_scan(temp_db.pool()).await,
            Err(OpenSubsonicError::Generic { source: _ })
        ));
    }
}
