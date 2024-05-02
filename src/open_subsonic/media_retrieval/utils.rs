use anyhow::Result;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::models::*;
use crate::open_subsonic::permission::with_permission;
use crate::{DatabasePool, OSError};

pub async fn get_song_download_info(
    pool: &DatabasePool,
    user_id: Uuid,
    song_id: Uuid,
) -> Result<(String, music_folders::FsType, String, i64, i32)> {
    songs::table
        .inner_join(music_folders::table)
        .filter(with_permission(user_id))
        .filter(songs::id.eq(song_id))
        .select((
            music_folders::path,
            music_folders::fs_type,
            songs::relative_path,
            songs::file_hash,
            songs::file_size,
        ))
        .first::<(String, music_folders::FsType, String, i64, i32)>(&mut pool.get().await?)
        .await
        .optional()?
        .ok_or_else(|| OSError::NotFound("Song".into()).into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_get_song_download_info_deny() {
        let mut infra = Infra::new().await.n_folder(2).await.add_user(None).await;
        infra.add_n_song(0, 1).await.add_n_song(1, 1).await.scan(.., None).await;
        infra.remove_permission(None, None).await.add_permissions(.., 1..).await;
        assert!(matches!(
            get_song_download_info(infra.pool(), infra.user_id(0), infra.song_ids(..1).await[0])
                .await
                .unwrap_err()
                .root_cause()
                .downcast_ref::<OSError>()
                .unwrap(),
            OSError::NotFound(_)
        ));
    }
}
