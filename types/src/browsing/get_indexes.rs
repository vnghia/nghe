use nghe_proc_macros::{add_common_convert, add_subsonic_response, add_types_derive};
use uuid::Uuid;

use super::super::common::id::MediaTypedId;

#[add_common_convert]
#[derive(Debug)]
pub struct GetIndexesParams {
    #[serde(rename = "musicFolderId")]
    pub music_folder_ids: Option<Vec<Uuid>>,
}

#[add_types_derive]
#[derive(Debug)]
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

#[add_types_derive]
pub struct Index {
    pub name: String,
    #[serde(rename = "artist")]
    pub children: Vec<ChildItem>,
}

#[add_types_derive]
pub struct Indexes {
    pub ignored_articles: String,
    pub index: Vec<Index>,
}

#[add_subsonic_response]
pub struct GetIndexesBody {
    pub indexes: Indexes,
}
