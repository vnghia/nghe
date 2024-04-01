use anyhow::Result;
use diesel::dsl::{exists, not, select};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::models::*;
use crate::{DatabasePool, OSError};

pub async fn check_permission(
    pool: &DatabasePool,
    user_id: Uuid,
    music_folder_ids: &Option<Vec<Uuid>>,
) -> Result<()> {
    if let Some(music_folder_ids) = music_folder_ids.as_ref()
        && !select(not(exists(
            user_music_folder_permissions::table
                .filter(user_music_folder_permissions::user_id.eq(user_id))
                .filter(user_music_folder_permissions::music_folder_id.eq_any(music_folder_ids))
                .filter(not(user_music_folder_permissions::allow)),
        )))
        .get_result::<bool>(&mut pool.get().await?)
        .await?
    {
        anyhow::bail!(OSError::Forbidden("access to these music folders".into()))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_check_permission_none() {
        let infra = Infra::new().await.n_folder(2).await.add_user(None).await;
        check_permission(infra.pool(), infra.user_id(0), &None).await.unwrap();
    }

    #[tokio::test]
    async fn test_check_permission_all() {
        let infra = Infra::new().await.n_folder(2).await.add_user(None).await;
        check_permission(infra.pool(), infra.user_id(0), &Some(infra.music_folder_ids(..)))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_check_permission_deny() {
        let infra = Infra::new().await.n_folder(2).await.add_user(None).await;
        infra.only_permissions(.., 1.., true).await;
        assert!(matches!(
            check_permission(infra.pool(), infra.user_id(0), &Some(infra.music_folder_ids(..)))
                .await
                .unwrap_err()
                .root_cause()
                .downcast_ref::<OSError>()
                .unwrap(),
            OSError::Forbidden(_)
        ));
    }
}
