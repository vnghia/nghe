use crate::entity::{prelude::*, *};

use itertools::Itertools;
use sea_orm::{DatabaseConnection, EntityTrait, *};

pub async fn refresh_user_music_folders(
    conn: &DatabaseConnection,
    music_folders: &[music_folder::Model],
) {
    let users = User::find()
        .select_column(user::Column::Id)
        .all(conn)
        .await
        .expect("can not get list of users");

    let user_music_folder_models =
        users
            .iter()
            .cartesian_product(music_folders)
            .map(|(user, music_folder)| user_music_folder::ActiveModel {
                user_id: Set(user.id),
                music_folder_id: Set(music_folder.id),
                ..Default::default()
            });

    UserMusicFolder::insert_many(user_music_folder_models)
        .on_conflict(
            sea_query::OnConflict::columns([
                user_music_folder::Column::UserId,
                user_music_folder::Column::MusicFolderId,
            ])
            .do_nothing()
            .to_owned(),
        )
        .on_empty_do_nothing()
        .exec(conn)
        .await
        .expect("can not set permission for in user music folder");

    tracing::info!("done refreshing user music folders");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test::{db::TemporaryDatabase, fs::TemporaryFs, user::create_db_users};

    async fn setup(
        u1d1: bool,
        u1d2: bool,
        u2d1: bool,
        u2d2: bool,
    ) -> (
        TemporaryDatabase,
        TemporaryFs,
        Vec<music_folder::Model>,
        Vec<user_music_folder::Model>,
    ) {
        let (db, user_tokens) = create_db_users(2, 0).await;
        let user1 = user_tokens[0].0.clone();
        let user2 = user_tokens[1].0.clone();

        let temp_fs = TemporaryFs::new();
        let upserted_folders = temp_fs.create_music_folders(db.get_conn(), 2).await;

        (
            db,
            temp_fs,
            upserted_folders.clone(),
            vec![
                user_music_folder::Model {
                    user_id: user1.id,
                    music_folder_id: upserted_folders[0].id,
                    allow: u1d1,
                },
                user_music_folder::Model {
                    user_id: user1.id,
                    music_folder_id: upserted_folders[1].id,
                    allow: u1d2,
                },
                user_music_folder::Model {
                    user_id: user2.id,
                    music_folder_id: upserted_folders[0].id,
                    allow: u2d1,
                },
                user_music_folder::Model {
                    user_id: user2.id,
                    music_folder_id: upserted_folders[1].id,
                    allow: u2d2,
                },
            ],
        )
    }

    fn sort_models(mut models: Vec<user_music_folder::Model>) -> Vec<user_music_folder::Model> {
        models.sort_by_key(|model| model.user_id);
        models.sort_by_key(|model| model.music_folder_id);
        models
    }

    #[tokio::test]
    async fn test_refresh_insert() {
        let (db, _temp_fs, music_folders, permissions) = setup(true, true, true, true).await;
        refresh_user_music_folders(db.get_conn(), &music_folders).await;

        let results = UserMusicFolder::find().all(db.get_conn()).await.unwrap();
        let permissions = sort_models(permissions);
        let results = sort_models(results);
        assert_eq!(permissions, results);

        db.async_drop().await;
    }

    #[tokio::test]
    async fn test_refresh_do_nothing() {
        let (db, _temp_fs, music_folders, permissions) = setup(true, false, true, true).await;
        db.insert(permissions[1].clone().into_active_model()).await;
        db.insert(permissions[3].clone().into_active_model()).await;
        refresh_user_music_folders(db.get_conn(), &music_folders).await;

        let results = UserMusicFolder::find().all(db.get_conn()).await.unwrap();
        let permissions = sort_models(permissions);
        let results = sort_models(results);
        assert_eq!(permissions, results);

        db.async_drop().await;
    }
}
