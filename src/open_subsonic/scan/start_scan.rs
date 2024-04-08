use std::borrow::Cow;

use anyhow::Result;
use axum::extract::State;
use axum::Extension;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{
    add_axum_response, add_common_convert, add_common_validate, add_subsonic_response,
};
use serde::Deserialize;
use time::OffsetDateTime;

use super::artist::build_artist_indices;
use super::run_scan::run_scan;
use crate::config::{ArtConfig, ArtistIndexConfig, ParsingConfig, ScanConfig};
use crate::models::*;
use crate::{Database, DatabasePool};

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum ScanMode {
    Full,
    Force,
}

#[add_common_convert]
#[derive(Debug)]
pub struct StartScanParams {
    scan_mode: ScanMode,
}
add_common_validate!(StartScanParams, admin);

#[add_subsonic_response]
pub struct StartScanBody {}
add_axum_response!(StartScanBody);

#[derive(Debug)]
#[cfg_attr(test, derive(Clone))]
pub struct ScanStatistic {
    pub scanned_song_count: usize,
    pub upserted_song_count: usize,
    pub deleted_song_count: usize,
    pub deleted_album_count: usize,
    pub deleted_artist_count: usize,
    pub deleted_genre_count: usize,
    pub scan_error_count: usize,
}

pub async fn initialize_scan(pool: &DatabasePool) -> Result<OffsetDateTime> {
    diesel::insert_into(scans::table)
        .default_values()
        .returning(scans::started_at)
        .get_result::<OffsetDateTime>(&mut pool.get().await?)
        .await
        .map_err(anyhow::Error::from)
}

pub async fn finalize_scan(
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
            scanned_count: scanned_count as _,
            error_message,
        })
        .execute(&mut pool.get().await?)
        .await?;
    Ok(())
}

pub async fn start_scan(
    pool: &DatabasePool,
    scan_mode: ScanMode,
    music_folders: &[music_folders::MusicFolder],
    artist_index_config: &ArtistIndexConfig,
    parsing_config: &ParsingConfig,
    scan_config: &ScanConfig,
    art_config: &ArtConfig,
) -> Result<ScanStatistic> {
    let scan_started_at = initialize_scan(pool).await?;
    let scan_result = run_scan(
        pool,
        scan_started_at,
        scan_mode,
        music_folders,
        parsing_config,
        scan_config,
        art_config,
    )
    .await;
    build_artist_indices(pool, artist_index_config).await?;
    finalize_scan(pool, scan_started_at, scan_result.as_ref()).await?;
    scan_result
}

pub async fn start_scan_handler(
    State(database): State<Database>,
    Extension(artist_index_config): Extension<ArtistIndexConfig>,
    Extension(parsing_config): Extension<ParsingConfig>,
    Extension(scan_config): Extension<ScanConfig>,
    Extension(art_config): Extension<ArtConfig>,
    req: StartScanRequest,
) -> StartScanJsonResponse {
    let music_folders = music_folders::table
        .select(music_folders::MusicFolder::as_select())
        .get_results(&mut database.pool.get().await?)
        .await?;
    tokio::task::spawn(async move {
        start_scan(
            &database.pool,
            req.params.scan_mode,
            &music_folders,
            &artist_index_config,
            &parsing_config,
            &scan_config,
            &art_config,
        )
        .await
    });
    StartScanBody {}.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test::TemporaryDb;

    #[tokio::test]
    async fn test_initialize_scan_twice() {
        let temp_db = TemporaryDb::new_from_env().await;
        initialize_scan(temp_db.pool()).await.unwrap();
        assert!(initialize_scan(temp_db.pool()).await.is_err());
    }
}
