use itertools::Itertools;

use crate::config::EncryptionKey;
use crate::entity::*;
use crate::utils::test::{db::TemporaryDatabase, fs::TemporaryFs, user::create_db_key_users};

pub async fn setup_user_and_music_folders(
    n_user: u8,
    n_folder: u8,
    allows: &[bool],
) -> (
    TemporaryDatabase,
    EncryptionKey,
    Vec<(user::Model, String, EncryptionKey)>,
    TemporaryFs,
    Vec<music_folder::Model>,
    Vec<user_music_folder::Model>,
) {
    let (db, key, user_tokens) = create_db_key_users(n_user, 0).await;
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
        .collect_vec();

    (
        db,
        key,
        user_tokens,
        temp_fs,
        upserted_folders,
        user_music_folders,
    )
}
