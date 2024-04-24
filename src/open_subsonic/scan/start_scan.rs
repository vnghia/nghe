use anyhow::Result;
use axum::extract::State;
use axum::Extension;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_axum_response, add_common_validate, add_convert_types};
use time::OffsetDateTime;
use uuid::Uuid;

use super::artist::insert_ignored_articles_config;
use super::get_scan_status::get_scan_status;
use super::run_scan::run_scan;
use crate::config::{ArtConfig, ArtistIndexConfig, ParsingConfig, ScanConfig};
use crate::models::*;
use crate::open_subsonic::lastfm::scan_artist_lastfm_info::scan_artist_lastfm_info;
use crate::{Database, DatabasePool};

add_common_validate!(StartScanParams, admin);
add_axum_response!(StartScanBody);

#[add_convert_types(into = scans::ScanStat)]
#[derive(Debug, Clone, Copy)]
pub struct ScanStat {
    pub scanned_song_count: usize,
    pub upserted_song_count: usize,
    pub deleted_song_count: usize,
    pub deleted_album_count: usize,
    pub deleted_artist_count: usize,
    pub deleted_genre_count: usize,
    pub scan_error_count: usize,
}

pub async fn initialize_scan(pool: &DatabasePool, id: Uuid) -> Result<OffsetDateTime> {
    diesel::insert_into(scans::table)
        .values(scans::NewScan { music_folder_id: id })
        .returning(scans::started_at)
        .get_result::<OffsetDateTime>(&mut pool.get().await?)
        .await
        .map_err(anyhow::Error::from)
}

pub async fn finalize_scan(
    pool: &DatabasePool,
    scan_started_at: OffsetDateTime,
    music_folder_id: Uuid,
    scan_result: Result<ScanStat>,
) -> Result<ScanStat> {
    match scan_result {
        Ok(scan_stat) => {
            diesel::update(
                scans::table
                    .filter(scans::started_at.eq(scan_started_at))
                    .filter(scans::music_folder_id.eq(music_folder_id)),
            )
            .set((
                scans::ScanStat::from(scan_stat),
                scans::is_scanning.eq(false),
                scans::finished_at.eq(OffsetDateTime::now_utc()),
                scans::unrecoverable.eq(false),
            ))
            .execute(&mut pool.get().await?)
            .await?;
            Ok(scan_stat)
        }
        r => {
            diesel::update(
                scans::table
                    .filter(scans::started_at.eq(scan_started_at))
                    .filter(scans::music_folder_id.eq(music_folder_id)),
            )
            .set((
                scans::is_scanning.eq(false),
                scans::finished_at.eq(OffsetDateTime::now_utc()),
                scans::unrecoverable.eq(true),
            ))
            .execute(&mut pool.get().await?)
            .await?;
            r
        }
    }
}

pub async fn start_scan(
    pool: &DatabasePool,
    scan_started_at: OffsetDateTime,
    params: StartScanParams,
    artist_index_config: &ArtistIndexConfig,
    parsing_config: &ParsingConfig,
    scan_config: &ScanConfig,
    art_config: &ArtConfig,
    lastfm_client: &Option<lastfm_client::Client>,
) -> Result<ScanStat> {
    let scan_result = run_scan(
        pool,
        scan_started_at,
        params.mode,
        music_folders::table
            .filter(music_folders::id.eq(params.id))
            .select(music_folders::MusicFolder::as_select())
            .get_result(&mut pool.get().await?)
            .await?,
        &artist_index_config.ignored_prefixes,
        parsing_config,
        scan_config,
        art_config,
    )
    .await;

    insert_ignored_articles_config(pool, &artist_index_config.ignored_articles).await?;
    if let Some(client) = lastfm_client {
        scan_artist_lastfm_info(pool, client, Some(scan_started_at)).await?;
    }

    finalize_scan(pool, scan_started_at, params.id, scan_result).await
}

pub async fn start_scan_handler(
    State(database): State<Database>,
    Extension(artist_index_config): Extension<ArtistIndexConfig>,
    Extension(parsing_config): Extension<ParsingConfig>,
    Extension(scan_config): Extension<ScanConfig>,
    Extension(art_config): Extension<ArtConfig>,
    Extension(lastfm_client): Extension<Option<lastfm_client::Client>>,
    req: StartScanRequest,
) -> StartScanJsonResponse {
    let id = req.params.id;
    let pool = database.pool.clone();
    let scan_started_at = initialize_scan(&pool, id).await?;

    tokio::task::spawn(async move {
        start_scan(
            &pool,
            scan_started_at,
            req.params,
            &artist_index_config,
            &parsing_config,
            &scan_config,
            &art_config,
            &lastfm_client,
        )
        .await
    });

    Ok(axum::Json(
        StartScanBody {
            scan: get_scan_status(&database.pool, id).await?.map(scans::ScanStatus::into),
        }
        .into(),
    ))
}

#[cfg(test)]
impl std::ops::Add for ScanStat {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            scanned_song_count: self.scanned_song_count + other.scanned_song_count,
            upserted_song_count: self.upserted_song_count + other.upserted_song_count,
            deleted_song_count: self.deleted_song_count + other.deleted_song_count,
            deleted_album_count: self.deleted_album_count + other.deleted_album_count,
            deleted_artist_count: self.deleted_artist_count + other.deleted_artist_count,
            deleted_genre_count: self.deleted_genre_count + other.deleted_genre_count,
            scan_error_count: self.scan_error_count + other.scan_error_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_initialize_scan_twice() {
        let infra = Infra::new().await.n_folder(1).await;

        initialize_scan(infra.pool(), infra.music_folder_id(0)).await.unwrap();
        assert!(initialize_scan(infra.pool(), infra.music_folder_id(0)).await.is_err());
    }
}
