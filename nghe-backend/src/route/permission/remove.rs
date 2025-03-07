use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
pub use nghe_api::permission::remove::{Request, Response};
use nghe_proc_macro::handler;

use crate::Error;
use crate::database::Database;
use crate::orm::{music_folders, user_music_folder_permissions, users};

#[handler(role = admin, internal = true)]
pub async fn handler(database: &Database, request: Request) -> Result<Response, Error> {
    let Request { user_id, music_folder_id } = request;

    if let Some(user_id) = user_id {
        if let Some(music_folder_id) = music_folder_id {
            diesel::delete(user_music_folder_permissions::table)
                .filter(user_music_folder_permissions::user_id.eq(user_id))
                .filter(user_music_folder_permissions::music_folder_id.eq(music_folder_id))
                .execute(&mut database.get().await?)
                .await?;
        } else {
            diesel::delete(user_music_folder_permissions::table)
                .filter(user_music_folder_permissions::user_id.eq(user_id))
                .filter(
                    user_music_folder_permissions::music_folder_id
                        .eq_any(music_folders::table.select(music_folders::id)),
                )
                .execute(&mut database.get().await?)
                .await?;
        }
    } else if let Some(music_folder_id) = music_folder_id {
        diesel::delete(user_music_folder_permissions::table)
            .filter(user_music_folder_permissions::user_id.eq_any(users::table.select(users::id)))
            .filter(user_music_folder_permissions::music_folder_id.eq(music_folder_id))
            .execute(&mut database.get().await?)
            .await?;
    } else {
        diesel::delete(user_music_folder_permissions::table)
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
    use crate::test::route::permission::count;
    use crate::test::{Mock, mock};

    #[rstest]
    #[case(true, true, 5)]
    #[case(true, false, 3)]
    #[case(false, true, 4)]
    #[case(false, false, 0)]
    #[tokio::test]
    async fn test_remove(
        #[future(awt)]
        #[with(2, 3)]
        mock: Mock,
        #[case] with_user: bool,
        #[case] with_music_folder: bool,
        #[case] permission_count: usize,
    ) {
        permission::add::handler(
            mock.database(),
            permission::add::Request { user_id: None, music_folder_id: None },
        )
        .await
        .unwrap();

        let user_id = if with_user { Some(mock.user_id(0).await) } else { None };
        let music_folder_id =
            if with_music_folder { Some(mock.music_folder_id(0).await) } else { None };
        assert!(handler(mock.database(), Request { user_id, music_folder_id }).await.is_ok());
        assert_eq!(count(&mock).await, permission_count);
    }
}
