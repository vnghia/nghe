use axum::extract::State;
use axum::Extension;
use diesel::{QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_validate, wrap_subsonic_response};

use super::{run_scan, ScanMode};
use crate::config::parsing::ParsingConfig;
use crate::config::{ArtistIndexConfig, ScanConfig};
use crate::models::*;
use crate::Database;

#[add_validate(admin = true)]
#[derive(Debug)]
pub struct StartScanParams {
    pub scan_mode: ScanMode,
}

#[wrap_subsonic_response]
pub struct StartScanBody {}

pub async fn start_scan_handler(
    State(database): State<Database>,
    Extension(artist_index_config): Extension<ArtistIndexConfig>,
    Extension(parsing_config): Extension<ParsingConfig>,
    Extension(scan_config): Extension<ScanConfig>,
    req: StartScanRequest,
) -> StartScanJsonResponse {
    let music_folders = music_folders::table
        .select(music_folders::MusicFolder::as_select())
        .get_results(&mut database.pool.get().await?)
        .await?;
    tokio::task::spawn(async move {
        run_scan(
            &database.pool,
            req.params.scan_mode,
            &music_folders,
            &artist_index_config,
            &parsing_config,
            &scan_config,
        )
        .await
    });
    StartScanBody {}.into()
}
