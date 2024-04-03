use axum::extract::State;
use itertools::Itertools;
use nghe_proc_macros::{add_validate, wrap_subsonic_response};
use serde::Serialize;
use uuid::Uuid;

use super::super::common::id::{MediaType, MediaTypedId};
use super::get_artists::get_artists;
use crate::Database;

#[add_validate]
#[derive(Debug)]
pub struct GetIndexesParams {
    #[serde(rename = "musicFolderId")]
    music_folder_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChildItem {
    pub id: MediaTypedId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<MediaTypedId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_dir: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_art: Option<MediaTypedId>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Index {
    pub name: String,
    #[serde(rename = "artist")]
    pub children: Vec<ChildItem>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Indexes {
    ignored_articles: String,
    index: Vec<Index>,
}

#[wrap_subsonic_response]
pub struct GetIndexesBody {
    indexes: Indexes,
}

pub async fn get_indexed_handler(
    State(database): State<Database>,
    req: GetIndexesRequest,
) -> GetIndexesJsonResponse {
    let indexed_artists =
        get_artists(&database.pool, req.user_id, &req.params.music_folder_ids).await?;
    let index = indexed_artists
        .index
        .into_iter()
        .sorted_by(|a, b| Ord::cmp(&a.name, &b.name))
        .map(|i| Index {
            name: i.name,
            children: i
                .artists
                .into_iter()
                .sorted_by(|a, b| Ord::cmp(&a.name, &b.name))
                .map(|c| ChildItem {
                    id: MediaTypedId { t: Some(MediaType::Aritst), id: c.id },
                    parent: None,
                    is_dir: None,
                    name: Some(c.name),
                    title: None,
                    cover_art: None,
                })
                .collect(),
        })
        .collect();
    GetIndexesBody {
        indexes: Indexes { ignored_articles: indexed_artists.ignored_articles, index },
    }
    .into()
}
