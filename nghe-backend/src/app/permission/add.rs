use diesel::{sql_types, IntoSql, JoinOnDsl, QueryDsl};
use diesel_async::RunQueryDsl;
pub use nghe_api::permission::add::{Request, Response};
use nghe_proc_macro::handler;

use crate::app::error::Error;
use crate::app::state::Database;
use crate::orm::{music_folders, user_music_folder_permissions, users};

#[handler(role = admin)]
pub async fn handler(database: &Database, request: Request) -> Result<Response, Error> {
    let Request { user_id, music_folder_id } = request;

    if let Some(user_id) = user_id {
        if let Some(music_folder_id) = music_folder_id {
            let new = user_music_folder_permissions::New { user_id, music_folder_id };

            diesel::insert_into(user_music_folder_permissions::table)
                .values(new)
                .on_conflict_do_nothing()
                .execute(&mut database.get().await?)
                .await?;
        } else {
            let new = music_folders::table
                .select((user_id.into_sql::<sql_types::Uuid>(), music_folders::schema::id));

            diesel::insert_into(user_music_folder_permissions::table)
                .values(new)
                .into_columns((
                    user_music_folder_permissions::schema::user_id,
                    user_music_folder_permissions::schema::music_folder_id,
                ))
                .on_conflict_do_nothing()
                .execute(&mut database.get().await?)
                .await?;
        }
    } else if let Some(music_folder_id) = music_folder_id {
        let new =
            users::table.select((users::schema::id, music_folder_id.into_sql::<sql_types::Uuid>()));

        diesel::insert_into(user_music_folder_permissions::table)
            .values(new)
            .into_columns((
                user_music_folder_permissions::schema::user_id,
                user_music_folder_permissions::schema::music_folder_id,
            ))
            .on_conflict_do_nothing()
            .execute(&mut database.get().await?)
            .await?;
    } else if cfg!(test) {
        let new = users::table
            .inner_join(music_folders::table.on(true.into_sql::<sql_types::Bool>()))
            .select((users::schema::id, music_folders::schema::id));

        diesel::insert_into(user_music_folder_permissions::table)
            .values(new)
            .into_columns((
                user_music_folder_permissions::schema::user_id,
                user_music_folder_permissions::schema::music_folder_id,
            ))
            .on_conflict_do_nothing()
            .execute(&mut database.get().await?)
            .await?;
    } else {
        return Err(Error::InvalidParameter(
            "The fields `user_id` and `music_folder_id` can not be both empty",
        ));
    }

    Ok(Response)
}
