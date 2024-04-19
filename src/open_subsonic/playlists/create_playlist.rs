use anyhow::Result;
use axum::extract::State;
use diesel_async::RunQueryDsl;
use futures::{stream, StreamExt, TryStreamExt};
use nghe_proc_macros::{add_axum_response, add_common_validate};
use nghe_types::playlists::create_playlist::CreatePlaylistParams;
use nghe_types::playlists::id3::*;
use uuid::Uuid;

use super::id3::*;
use super::utils::{add_songs, get_playlist_and_songs};
use crate::models::*;
use crate::open_subsonic::id3::*;
use crate::{Database, DatabasePool, OSError};

add_common_validate!(CreatePlaylistParams);
add_axum_response!(CreatePlaylistBody);

pub async fn create_playlist(
    pool: &DatabasePool,
    user_id: Uuid,
    CreatePlaylistParams { name, playlist_id, song_ids }: &CreatePlaylistParams,
) -> Result<(PlaylistId3Db, Vec<Uuid>)> {
    let playlist_id = if let Some(name) = name.as_ref() {
        let playlist_id = diesel::insert_into(playlists::table)
            .values(playlists::NewPlaylist { name: name.into() })
            .returning(playlists::id)
            .get_result::<Uuid>(&mut pool.get().await?)
            .await?;
        diesel::insert_into(playlists_users::table)
            .values(playlists_users::AddUser {
                playlist_id,
                user_id,
                access_level: playlists_users::AccessLevel::Admin,
            })
            .execute(&mut pool.get().await?)
            .await?;
        playlist_id
    } else {
        playlist_id.ok_or_else(|| {
            OSError::InvalidParameter("either name or playlist id must be specified".into())
        })?
    };

    if song_ids.is_empty() {
        add_songs(pool, playlist_id, song_ids).await?;
    }

    get_playlist_and_songs(pool, user_id, playlist_id).await
}

pub async fn create_playlist_handler(
    State(database): State<Database>,
    req: CreatePlaylistRequest,
) -> CreatePlaylistJsonResponse {
    let pool = &database.pool;

    let (playlist, song_ids) = create_playlist(pool, req.user_id, &req.params).await?;
    let songs = get_songs(pool, &song_ids).await?;

    Ok(axum::Json(
        CreatePlaylistBody {
            playlist: PlaylistId3WithSongs {
                playlist: playlist.into(),
                songs: stream::iter(songs)
                    .then(|v| async move { v.into(pool).await })
                    .try_collect()
                    .await?,
            },
        }
        .into(),
    ))
}
