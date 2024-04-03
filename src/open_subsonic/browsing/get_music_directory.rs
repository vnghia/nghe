use anyhow::Result;
use axum::extract::State;
use itertools::Itertools;
use nghe_proc_macros::{add_validate, wrap_subsonic_response};
use serde::Serialize;
use uuid::Uuid;

use super::super::common::id::{MediaType, MediaTypedId};
use super::get_indexes::ChildItem;
use crate::open_subsonic::browsing::get_album::get_album;
use crate::open_subsonic::browsing::get_artist::get_artist;
use crate::{Database, DatabasePool, OSError};

#[add_validate]
#[derive(Debug)]
pub struct GetMusicDirectoryParams {
    id: MediaTypedId,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicDirectory {
    id: MediaTypedId,
    name: String,
    #[serde(rename = "child")]
    children: Vec<ChildItem>,
}

#[wrap_subsonic_response]
pub struct GetMusicDirectoryBody {
    directory: MusicDirectory,
}

async fn get_artist_directory(
    pool: &DatabasePool,
    user_id: Uuid,
    parent_id: MediaTypedId,
) -> Result<MusicDirectory> {
    let artist = get_artist(pool, user_id, parent_id.id).await?;
    let children = artist
        .albums
        .into_iter()
        .sorted_by(|a, b| Ord::cmp(&a.name, &b.name))
        .map(|a| ChildItem {
            id: MediaTypedId { t: Some(MediaType::Album), id: a.id },
            parent: Some(parent_id),
            is_dir: Some(true),
            name: None,
            title: Some(a.name),
            cover_art: Some(a.cover_art),
        })
        .collect();

    Ok(MusicDirectory { id: parent_id, name: artist.artist.name, children })
}

async fn get_album_directory(
    pool: &DatabasePool,
    user_id: Uuid,
    parent_id: MediaTypedId,
) -> Result<MusicDirectory> {
    let album = get_album(pool, user_id, parent_id.id).await?;
    let children = album
        .songs
        .into_iter()
        .sorted_by(|a, b| Ord::cmp(&a.title, &b.title))
        .map(|s| ChildItem {
            id: MediaTypedId { t: None, id: s.id },
            parent: Some(parent_id),
            is_dir: Some(false),
            name: None,
            title: Some(s.title),
            cover_art: s.cover_art,
        })
        .collect();

    Ok(MusicDirectory { id: parent_id, name: album.album.name, children })
}

pub async fn get_music_directory_handler(
    State(database): State<Database>,
    req: GetMusicDirectoryRequest,
) -> GetMusicDirectoryJsonResponse {
    GetMusicDirectoryBody {
        directory: match req.params.id.t {
            Some(MediaType::Aritst) => {
                get_artist_directory(&database.pool, req.user_id, req.params.id).await?
            }
            Some(MediaType::Album) => {
                get_album_directory(&database.pool, req.user_id, req.params.id).await?
            }
            _ => Err(anyhow::anyhow!(OSError::NotFound("Music directory".into())))?,
        },
    }
    .into()
}
