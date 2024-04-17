use anyhow::Result;
use axum::extract::State;
use diesel::{
    ExpressionMethods, OptionalExtension, PgSortExpressionMethods, QueryDsl, SelectableHelper,
};
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_axum_response, add_common_validate};
use uuid::Uuid;

use crate::models::*;
use crate::{Database, DatabasePool};

add_common_validate!(GetScanStatusParams, admin);
add_axum_response!(GetScanStatusBody);

pub async fn get_scan_status(pool: &DatabasePool, id: Uuid) -> Result<Option<scans::ScanStatus>> {
    scans::table
        .select(scans::ScanStatus::as_select())
        .filter(scans::music_folder_id.eq(id))
        .order(scans::started_at.desc().nulls_last())
        .first(&mut pool.get().await?)
        .await
        .optional()
        .map_err(anyhow::Error::from)
}

pub async fn get_scan_status_handler(
    State(database): State<Database>,
    req: GetScanStatusRequest,
) -> GetScanStatusJsonResponse {
    Ok(axum::Json(
        GetScanStatusBody {
            status: get_scan_status(&database.pool, req.params.id)
                .await?
                .map(scans::ScanStatus::into),
        }
        .into(),
    ))
}
