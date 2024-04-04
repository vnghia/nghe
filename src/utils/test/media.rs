use diesel::QueryDsl;
use diesel_async::RunQueryDsl;
use itertools::Itertools;

use super::fs::SongFsInformation;
use crate::models::*;
use crate::DatabasePool;

pub async fn assert_albums_info(pool: &DatabasePool, song_fs_infos: &[SongFsInformation]) {
    assert_album_names(
        pool,
        &song_fs_infos.iter().map(|s| s.tag.album.clone()).unique().sorted().collect_vec(),
    )
    .await;
}

pub async fn assert_album_names<S: AsRef<str>>(pool: &DatabasePool, names: &[S]) {
    assert_eq!(
        names.iter().map(|name| name.as_ref()).unique().sorted().collect_vec(),
        albums::table
            .select(albums::name)
            .load::<String>(&mut pool.get().await.unwrap())
            .await
            .unwrap()
            .iter()
            .map(std::string::String::as_str)
            .sorted()
            .collect_vec(),
    );
}
