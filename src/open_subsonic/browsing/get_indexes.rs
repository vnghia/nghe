use std::str::FromStr;

use anyhow::Result;
use axum::extract::State;
use itertools::Itertools;
use nghe_proc_macros::{add_validate, wrap_subsonic_response};
use serde::Serialize;
use uuid::Uuid;

use super::super::common::id::TypedId;
use super::get_artists::get_artists;
use crate::{Database, OSError};

#[add_validate]
#[derive(Debug)]
pub struct GetIndexesParams {
    #[serde(rename = "musicFolderId")]
    music_folder_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Clone, Copy)]
pub enum DirectoryType {
    Aritst,
    Album,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChildItem {
    pub id: TypedId<DirectoryType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<TypedId<DirectoryType>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_dir: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>
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
        get_artists(&database.pool, req.user_id, req.params.music_folder_ids).await?;
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
                    id: TypedId { t: Some(DirectoryType::Aritst), id: c.id },
                    parent: None,
                    is_dir: None,
                    name: Some(c.name),
                    title: None,
                })
                .collect(),
        })
        .collect();
    GetIndexesBody {
        indexes: Indexes { ignored_articles: indexed_artists.ignored_articles, index },
    }
    .into()
}

impl AsRef<str> for DirectoryType {
    fn as_ref(&self) -> &'static str {
        match self {
            DirectoryType::Aritst => "ar",
            DirectoryType::Album => "al",
        }
    }
}

impl FromStr for DirectoryType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "ar" => Ok(DirectoryType::Aritst),
            "al" => Ok(DirectoryType::Album),
            _ => anyhow::bail!(OSError::InvalidParameter(
                concat_string::concat_string!("Value passed to enum DirectoryType {}", s).into()
            )),
        }
    }
}
