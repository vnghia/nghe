use diesel::dsl::exists;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use itertools::Itertools;

use super::fs::SongFsInformation;
use crate::models::*;
use crate::DatabasePool;

pub async fn assert_artists_info(pool: &DatabasePool, song_fs_infos: &[SongFsInformation]) {
    assert_artist_names(
        pool,
        &song_fs_infos
            .iter()
            .flat_map(|s| s.tag.album_artists.iter().chain(s.tag.artists.iter()).collect_vec())
            .unique()
            .sorted()
            .collect_vec(),
    )
    .await;
}

pub async fn assert_albums_artists_info(pool: &DatabasePool, song_fs_infos: &[SongFsInformation]) {
    assert_album_artist_names(
        pool,
        &song_fs_infos
            .iter()
            .flat_map(|s| s.tag.album_artists_or_default())
            .unique()
            .sorted()
            .collect_vec(),
    )
    .await;
}

pub async fn assert_albums_info(pool: &DatabasePool, song_fs_infos: &[SongFsInformation]) {
    assert_album_names(
        pool,
        &song_fs_infos.iter().map(|s| s.tag.album.clone()).unique().sorted().collect_vec(),
    )
    .await;
}

pub async fn assert_artist_names<S: AsRef<str>>(pool: &DatabasePool, names: &[S]) {
    assert_eq!(
        names.iter().map(|name| name.as_ref()).unique().sorted().collect_vec(),
        artists::table
            .select(artists::name)
            .load::<String>(&mut pool.get().await.unwrap())
            .await
            .unwrap()
            .iter()
            .map(std::string::String::as_str)
            .sorted()
            .collect_vec(),
    );
}

pub async fn assert_song_artist_names<S: AsRef<str>>(pool: &DatabasePool, names: &[S]) {
    assert_eq!(
        names.iter().map(|name| name.as_ref()).unique().sorted().collect_vec(),
        artists::table
            .filter(exists(songs_artists::table.filter(songs_artists::artist_id.eq(artists::id))))
            .select(artists::name)
            .distinct()
            .load::<String>(&mut pool.get().await.unwrap())
            .await
            .unwrap()
            .iter()
            .map(std::string::String::as_str)
            .sorted()
            .collect_vec(),
    );
}

pub async fn assert_album_artist_names<S: AsRef<str>>(pool: &DatabasePool, names: &[S]) {
    assert_eq!(
        names.iter().map(|name| name.as_ref()).unique().sorted().collect_vec(),
        artists::table
            .filter(exists(
                songs_album_artists::table
                    .filter(songs_album_artists::album_artist_id.eq(artists::id))
            ))
            .select(artists::name)
            .distinct()
            .load::<String>(&mut pool.get().await.unwrap())
            .await
            .unwrap()
            .iter()
            .map(std::string::String::as_str)
            .sorted()
            .collect_vec(),
    );
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
