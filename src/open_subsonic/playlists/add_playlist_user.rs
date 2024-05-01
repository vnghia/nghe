use anyhow::Result;
use axum::extract::State;
use nghe_proc_macros::{add_axum_response, add_common_validate};
use nghe_types::playlists::add_playlist_user::AddPlaylistUserParams;
use uuid::Uuid;

use super::utils::{add_playlist_user_unchecked, check_access_level};
use crate::models::*;
use crate::{Database, DatabasePool};

add_common_validate!(AddPlaylistUserParams);
add_axum_response!(AddPlaylistUserBody);

pub async fn add_playlist_user(
    pool: &DatabasePool,
    admin_id: Uuid,
    playlist_id: Uuid,
    user_id: Uuid,
    access_level: playlists_users::AccessLevel,
) -> Result<()> {
    check_access_level(pool, playlist_id, admin_id, playlists_users::AccessLevel::Admin).await?;
    add_playlist_user_unchecked(pool, playlist_id, user_id, access_level).await?;
    Ok(())
}

pub async fn add_playlist_user_handler(
    State(database): State<Database>,
    req: AddPlaylistUserRequest,
) -> AddPlaylistUserJsonResponse {
    add_playlist_user(
        &database.pool,
        req.user_id,
        req.params.playlist_id,
        req.params.user_id,
        req.params.access_level.into(),
    )
    .await?;
    Ok(axum::Json(AddPlaylistUserBody {}.into()))
}

#[cfg(test)]
mod tests {
    use nghe_types::playlists::create_playlist::CreatePlaylistParams;

    use super::super::create_playlist::create_playlist;
    use super::*;
    use crate::utils::test::Infra;
    use crate::OSError;

    #[tokio::test]
    async fn test_add_playlist_user() {
        let infra = Infra::new().await.add_user(None).await.add_user(None).await.n_folder(1).await;
        let playlist_id = create_playlist(
            infra.pool(),
            infra.user_id(0),
            &CreatePlaylistParams {
                name: Some("playlist".into()),
                playlist_id: None,
                song_ids: None,
            },
        )
        .await
        .unwrap()
        .playlist
        .basic
        .id;

        add_playlist_user(
            infra.pool(),
            infra.user_id(0),
            playlist_id,
            infra.user_id(1),
            playlists_users::AccessLevel::Read,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_add_playlist_user_deny() {
        let infra = Infra::new()
            .await
            .add_user(None)
            .await
            .add_user(None)
            .await
            .add_user(None)
            .await
            .n_folder(1)
            .await;
        let playlist_id = create_playlist(
            infra.pool(),
            infra.user_id(0),
            &CreatePlaylistParams {
                name: Some("playlist".into()),
                playlist_id: None,
                song_ids: None,
            },
        )
        .await
        .unwrap()
        .playlist
        .basic
        .id;

        add_playlist_user(
            infra.pool(),
            infra.user_id(0),
            playlist_id,
            infra.user_id(1),
            playlists_users::AccessLevel::Read,
        )
        .await
        .unwrap();

        assert!(matches!(
            add_playlist_user(
                infra.pool(),
                infra.user_id(1),
                playlist_id,
                infra.user_id(2),
                playlists_users::AccessLevel::Read
            )
            .await
            .unwrap_err()
            .root_cause()
            .downcast_ref::<OSError>()
            .unwrap(),
            OSError::Forbidden(_)
        ))
    }
}
