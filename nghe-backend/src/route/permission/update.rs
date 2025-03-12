use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
pub use nghe_api::permission::update::{Request, Response};
use nghe_proc_macro::handler;

use crate::Error;
use crate::database::Database;
use crate::orm::user_music_folder_permissions;

#[handler(role = admin, internal = true)]
pub async fn handler(database: &Database, request: Request) -> Result<Response, Error> {
    let Request { user_id, music_folder_id, permission } = request;
    let permission: user_music_folder_permissions::Permission = permission.into();

    if let Some(user_id) = user_id {
        if let Some(music_folder_id) = music_folder_id {
            diesel::update(user_music_folder_permissions::table)
                .filter(user_music_folder_permissions::user_id.eq(user_id))
                .filter(user_music_folder_permissions::music_folder_id.eq(music_folder_id))
                .set(permission)
                .execute(&mut database.get().await?)
                .await?;
        } else {
            diesel::update(user_music_folder_permissions::table)
                .filter(user_music_folder_permissions::user_id.eq(user_id))
                .set(permission)
                .execute(&mut database.get().await?)
                .await?;
        }
    } else if let Some(music_folder_id) = music_folder_id {
        diesel::update(user_music_folder_permissions::table)
            .filter(user_music_folder_permissions::music_folder_id.eq(music_folder_id))
            .set(permission)
            .execute(&mut database.get().await?)
            .await?;
    } else {
        tracing::warn!("Updating permission for all users with all music folders");

        diesel::update(user_music_folder_permissions::table)
            .set(permission)
            .execute(&mut database.get().await?)
            .await?;
    }

    Ok(Response)
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::route::permission;
    use crate::test::route::permission::count_owner;
    use crate::test::{Mock, mock};

    #[rstest]
    #[case(true, true, 1)]
    #[case(true, false, 3)]
    #[case(false, true, 2)]
    #[case(false, false, 6)]
    #[tokio::test]
    async fn test_handler(
        #[future(awt)]
        #[with(2, 3)]
        mock: Mock,
        #[case] with_user: bool,
        #[case] with_music_folder: bool,
        #[case] permission_count: usize,
    ) {
        permission::add::handler(
            mock.database(),
            permission::add::Request {
                user_id: None,
                music_folder_id: None,
                permission: nghe_api::permission::Permission::default(),
            },
        )
        .await
        .unwrap();

        let user_id = if with_user { Some(mock.user_id(0).await) } else { None };
        let music_folder_id =
            if with_music_folder { Some(mock.music_folder_id(0).await) } else { None };
        assert!(
            handler(
                mock.database(),
                Request {
                    user_id,
                    music_folder_id,
                    permission: nghe_api::permission::Permission {
                        owner: true,
                        ..nghe_api::permission::Permission::default()
                    }
                }
            )
            .await
            .is_ok()
        );
        assert_eq!(count_owner(&mock).await, permission_count);
    }
}
