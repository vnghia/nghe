use super::get_artists::{get_artists, Indexes};
use crate::Database;

use axum::extract::State;
use nghe_proc_macros::{add_validate, wrap_subsonic_response};
use uuid::Uuid;

#[add_validate]
#[derive(Debug)]
pub struct GetIndexesParams {
    #[serde(rename = "musicFolderId")]
    music_folder_ids: Option<Vec<Uuid>>,
}

#[wrap_subsonic_response]
pub struct GetIndexesBody {
    indexes: Indexes,
}

pub async fn get_indexed_handler(
    State(database): State<Database>,
    req: GetIndexesRequest,
) -> GetIndexesJsonResponse {
    GetIndexesBody {
        indexes: get_artists(&database.pool, req.user_id, req.params.music_folder_ids).await?,
    }
    .into()
}
