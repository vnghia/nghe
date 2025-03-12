use diesel::{IntoSql, JoinOnDsl, QueryDsl, sql_types};
use diesel_async::RunQueryDsl;
pub use nghe_api::permission::add::{Request, Response};
use nghe_proc_macro::handler;

use crate::Error;
use crate::database::Database;
use crate::orm::{music_folders, user_music_folder_permissions, users};

#[handler(role = admin, internal = true)]
pub async fn handler(database: &Database, request: Request) -> Result<Response, Error> {
    let Request { user_id, music_folder_id, permission } = request;
    let permission = permission.into();

    if let Some(user_id) = user_id {
        if let Some(music_folder_id) = music_folder_id {
            let new = user_music_folder_permissions::New { user_id, music_folder_id, permission };

            diesel::insert_into(user_music_folder_permissions::table)
                .values(new)
                .on_conflict_do_nothing()
                .execute(&mut database.get().await?)
                .await?;
        } else {
            let new = music_folders::table
                .select((user_id.into_sql::<sql_types::Uuid>(), music_folders::id));

            diesel::insert_into(user_music_folder_permissions::table)
                .values(new)
                .into_columns((
                    user_music_folder_permissions::user_id,
                    user_music_folder_permissions::music_folder_id,
                ))
                .on_conflict_do_nothing()
                .execute(&mut database.get().await?)
                .await?;
        }
    } else if let Some(music_folder_id) = music_folder_id {
        let new = users::table.select((users::id, music_folder_id.into_sql::<sql_types::Uuid>()));

        diesel::insert_into(user_music_folder_permissions::table)
            .values(new)
            .into_columns((
                user_music_folder_permissions::user_id,
                user_music_folder_permissions::music_folder_id,
            ))
            .on_conflict_do_nothing()
            .execute(&mut database.get().await?)
            .await?;
    } else {
        tracing::warn!("Adding permission for all users with all music folders");

        let new = users::table
            .inner_join(music_folders::table.on(true.into_sql::<sql_types::Bool>()))
            .select((users::id, music_folders::id));

        diesel::insert_into(user_music_folder_permissions::table)
            .values(new)
            .into_columns((
                user_music_folder_permissions::user_id,
                user_music_folder_permissions::music_folder_id,
            ))
            .on_conflict_do_nothing()
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
    use crate::test::route::permission::{count, reset};
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
        reset(&mock).await;
        let user_id = if with_user { Some(mock.user_id(0).await) } else { None };
        let music_folder_id =
            if with_music_folder { Some(mock.music_folder_id(0).await) } else { None };
        assert!(
            handler(
                mock.database(),
                Request {
                    user_id,
                    music_folder_id,
                    permission: nghe_api::permission::Permission::default()
                }
            )
            .await
            .is_ok()
        );
        assert_eq!(count(&mock).await, permission_count);
    }
}
