use crate::models::*;
use crate::utils::song::tag::SongTag;
use crate::DatabasePool;

use diesel::{dsl::exists, ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use futures::stream::{self, StreamExt};
use itertools::Itertools;
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

pub async fn query_all_song_information(
    pool: &DatabasePool,
    song_id: Uuid,
) -> (
    songs::Song,
    albums::Album,
    Vec<artists::Artist>,
    Vec<artists::Artist>,
) {
    let song = songs::table
        .filter(songs::id.eq(song_id))
        .select(songs::Song::as_select())
        .first(
            &mut pool
                .get()
                .await
                .expect("can not check out connection to the database"),
        )
        .await
        .expect("can not query song");

    let album = albums::table
        .filter(albums::id.eq(song.album_id))
        .select(albums::Album::as_select())
        .first(
            &mut pool
                .get()
                .await
                .expect("can not check out connection to the database"),
        )
        .await
        .expect("can not query album");

    let artists = artists::table
        .inner_join(songs_artists::table)
        .filter(songs_artists::song_id.eq(song_id))
        .select(artists::Artist::as_select())
        .get_results(
            &mut pool
                .get()
                .await
                .expect("can not check out connection to the database"),
        )
        .await
        .expect("can not query song artists");

    let album_artists = songs_album_artists::table
        .inner_join(artists::table)
        .inner_join(songs::table)
        .filter(songs::album_id.eq(album.id))
        .select(artists::Artist::as_select())
        .get_results(
            &mut pool
                .get()
                .await
                .expect("can not check out connection to the database"),
        )
        .await
        .expect("can not query song artists");

    (song, album, artists, album_artists)
}

pub async fn query_all_songs_information(
    pool: &DatabasePool,
) -> HashMap<
    (Uuid, PathBuf),
    (
        songs::Song,
        albums::Album,
        Vec<artists::Artist>,
        Vec<artists::Artist>,
    ),
> {
    let song_ids = songs::table
        .select(songs::id)
        .get_results(
            &mut pool
                .get()
                .await
                .expect("can not check out connection to the database"),
        )
        .await
        .expect("can not query song ids");
    stream::iter(song_ids)
        .then(|song_id| async move {
            let result = query_all_song_information(pool, song_id).await;
            (
                (result.0.music_folder_id, PathBuf::from(&result.0.path)),
                result,
            )
        })
        .collect::<HashMap<_, _>>()
        .await
}

pub async fn assert_artists_info(
    pool: &DatabasePool,
    song_fs_info: &HashMap<(Uuid, PathBuf), SongTag>,
) {
    assert_artist_names(
        pool,
        &song_fs_info
            .values()
            .flat_map(|song_tag| {
                song_tag
                    .album_artists
                    .iter()
                    .chain(song_tag.artists.iter())
                    .collect_vec()
            })
            .unique()
            .sorted()
            .collect_vec(),
    )
    .await;
}

pub async fn assert_albums_artists_info(
    pool: &DatabasePool,
    song_fs_info: &HashMap<(Uuid, PathBuf), SongTag>,
) {
    assert_album_artist_names(
        pool,
        &song_fs_info
            .values()
            .flat_map(|song_tag| song_tag.album_artists.clone())
            .unique()
            .sorted()
            .collect_vec(),
    )
    .await;
}

pub async fn assert_albums_info(
    pool: &DatabasePool,
    song_fs_info: &HashMap<(Uuid, PathBuf), SongTag>,
) {
    assert_album_names(
        pool,
        &song_fs_info
            .values()
            .map(|song_tag| song_tag.album.clone())
            .unique()
            .sorted()
            .collect_vec(),
    )
    .await;
}

pub async fn assert_songs_info(
    pool: &DatabasePool,
    song_fs_info: &HashMap<(Uuid, PathBuf), SongTag>,
) {
    let mut song_db_info = query_all_songs_information(pool).await;

    for (song_key, song_tag) in song_fs_info {
        let (song, album, artists, album_artists) = song_db_info.remove(song_key).unwrap();
        assert_eq!(song_tag.title, song.title);
        assert_eq!(song_tag.album, album.name);
        assert_eq!(
            song_tag.artists.iter().sorted().collect_vec(),
            artists
                .iter()
                .map(|artist| &artist.name)
                .sorted()
                .collect_vec()
        );
        assert_eq!(
            song_tag.album_artists.iter().sorted().collect_vec(),
            album_artists
                .iter()
                .map(|artist| &artist.name)
                .sorted()
                .collect_vec()
        );
    }
    assert!(song_db_info.is_empty());
}

pub async fn assert_artist_names<S: AsRef<str>>(pool: &DatabasePool, names: &[S]) {
    assert_eq!(
        names
            .iter()
            .map(|name| name.as_ref())
            .unique()
            .sorted()
            .collect_vec(),
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
        names
            .iter()
            .map(|name| name.as_ref())
            .unique()
            .sorted()
            .collect_vec(),
        artists::table
            .filter(exists(
                songs_artists::table.filter(songs_artists::artist_id.eq(artists::id))
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

pub async fn assert_album_artist_names<S: AsRef<str>>(pool: &DatabasePool, names: &[S]) {
    assert_eq!(
        names
            .iter()
            .map(|name| name.as_ref())
            .unique()
            .sorted()
            .collect_vec(),
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
        names
            .iter()
            .map(|name| name.as_ref())
            .unique()
            .sorted()
            .collect_vec(),
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

pub async fn song_paths_to_ids(
    pool: &DatabasePool,
    song_fs_info: &HashMap<(Uuid, PathBuf), SongTag>,
) -> Vec<Uuid> {
    stream::iter(song_fs_info.keys())
        .then(|(music_folder_id, path)| async move {
            songs::table
                .select(songs::id)
                .filter(songs::music_folder_id.eq(music_folder_id))
                .filter(songs::path.eq(path.to_str().unwrap()))
                .first::<Uuid>(&mut pool.get().await.unwrap())
                .await
                .unwrap()
        })
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .sorted()
        .collect_vec()
}

pub async fn song_paths_to_album_ids(
    pool: &DatabasePool,
    song_fs_info: &HashMap<(Uuid, PathBuf), SongTag>,
) -> Vec<Uuid> {
    stream::iter(song_fs_info.keys())
        .then(|(music_folder_id, path)| async move {
            songs::table
                .select(songs::album_id)
                .filter(songs::music_folder_id.eq(music_folder_id))
                .filter(songs::path.eq(path.to_str().unwrap()))
                .first::<Uuid>(&mut pool.get().await.unwrap())
                .await
                .unwrap()
        })
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .unique()
        .sorted()
        .collect_vec()
}
