use crate::models::*;
use crate::{DatabasePool, OSResult};

use diesel::SelectableHelper;
use diesel_async::RunQueryDsl;
use std::borrow::Cow;
use std::path::Path;
use uuid::Uuid;

#[allow(clippy::too_many_arguments)]
pub async fn insert_song<'a, TM: AsRef<Path>, TP: AsRef<Path>>(
    pool: &DatabasePool,
    title: Cow<'a, str>,
    album_id: Uuid,
    music_folder_id: Uuid,
    music_folder_path: TM,
    file_path: TP,
    file_hash: u64,
    file_size: u64,
) -> OSResult<songs::Song> {
    Ok(diesel::insert_into(songs::table)
        .values(&songs::NewSong {
            title,
            album_id,
            music_folder_id,
            path: file_path
                .as_ref()
                .strip_prefix(music_folder_path.as_ref())?
                .to_string_lossy(),
            file_hash: file_hash as i64,
            file_size: file_size as i64,
        })
        .on_conflict_do_nothing()
        .returning(songs::Song::as_returning())
        .get_result(&mut pool.get().await?)
        .await?)
}
