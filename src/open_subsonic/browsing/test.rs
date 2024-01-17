use itertools::Itertools;

use crate::entity::*;
use crate::utils::test::{db::TemporaryDatabase, fs::TemporaryFs, user::create_db_users};

pub async fn setup_user_and_music_folders(
    n_user: u8,
    n_folder: u8,
    allows: &[bool],
) -> (
    TemporaryDatabase,
    TemporaryFs,
    Vec<music_folder::Model>,
    Vec<user_music_folder::Model>,
) {
    let (db, user_tokens) = create_db_users(n_user, 0).await;
    let temp_fs = TemporaryFs::new();
    let upserted_folders = temp_fs.create_music_folders(db.get_conn(), n_folder).await;
    let user_music_folders = (user_tokens.iter().cartesian_product(&upserted_folders))
        .zip(allows.iter())
        .map(
            |((user_token, upserted_folder), allow)| user_music_folder::Model {
                user_id: user_token.0.id,
                music_folder_id: upserted_folder.id,
                allow: *allow,
            },
        )
        .collect::<Vec<_>>();

    (db, temp_fs, upserted_folders.clone(), user_music_folders)
}
