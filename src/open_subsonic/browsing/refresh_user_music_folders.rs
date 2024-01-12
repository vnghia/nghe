use crate::entity::{prelude::*, *};

use concat_string::concat_string;
use futures::stream::{self, StreamExt};
use sea_orm::{DatabaseConnection, EntityTrait, *};

pub async fn refresh_user_music_folders(
    conn: &DatabaseConnection,
    music_folders: &[music_folder::Model],
) {
    let users = user::Entity::find()
        .select_column(user::Column::Id)
        .all(conn)
        .await
        .expect("can not get list of users");

    stream::iter(users)
        .for_each(|user| async move {
            stream::iter(music_folders)
                .for_each(move |music_folder| async move {
                    UserMusicFolder::insert(user_music_folder::ActiveModel {
                        user_id: Set(user.id),
                        music_folder_id: Set(music_folder.id),
                        ..Default::default()
                    })
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
                    .expect(&concat_string!(
                        "can not set permission for user with folder ",
                        &music_folder.path
                    ));
                })
                .await;
        })
        .await;
    tracing::info!("done refreshing user music folders");
}

#[cfg(test)]
mod tests {
    use super::super::super::{
        browsing::refresh_music_folders,
        user::create::{create_user, CreateUserParams},
    };
    use super::*;
    use crate::utils::test::{db::TemporaryDatabase, fs::TemporaryFs, user::create_key_user_token};

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
        let db = TemporaryDatabase::new_from_env().await;
        let (key, _, _, _, _) = create_key_user_token();

        let (_, username1, password1, _, _) = create_key_user_token();
        let user1 = create_user(
            db.get_conn(),
            &key,
            CreateUserParams {
                username: username1,
                password: password1,
                ..Default::default()
            },
        )
        .await
        .unwrap();

        let (_, username2, password2, _, _) = create_key_user_token();
        let user2 = create_user(
            db.get_conn(),
            &key,
            CreateUserParams {
                username: username2,
                password: password2,
                ..Default::default()
            },
        )
        .await
        .unwrap();

        let temp_fs = TemporaryFs::new();
        let dir_1 = temp_fs.create_dir("test1/").await;
        let dir_2 = temp_fs.create_dir("test2/").await;

        let (mut upserted_folders, _) =
            refresh_music_folders(db.get_conn(), &[dir_1, dir_2], &[]).await;
        upserted_folders.sort_by_key(|model| model.path.clone());

        let dir_ids = upserted_folders
            .iter()
            .map(|model| model.id)
            .collect::<Vec<_>>();

        (
            db,
            temp_fs,
            upserted_folders,
            vec![
                user_music_folder::Model {
                    user_id: user1.id,
                    music_folder_id: dir_ids[0],
                    allow: u1d1,
                },
                user_music_folder::Model {
                    user_id: user1.id,
                    music_folder_id: dir_ids[1],
                    allow: u1d2,
                },
                user_music_folder::Model {
                    user_id: user2.id,
                    music_folder_id: dir_ids[0],
                    allow: u2d1,
                },
                user_music_folder::Model {
                    user_id: user2.id,
                    music_folder_id: dir_ids[1],
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

        let results = user_music_folder::Entity::find()
            .all(db.get_conn())
            .await
            .unwrap();
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

        let results = user_music_folder::Entity::find()
            .all(db.get_conn())
            .await
            .unwrap();
        let permissions = sort_models(permissions);
        let results = sort_models(results);
        assert_eq!(permissions, results);

        db.async_drop().await;
    }
}
