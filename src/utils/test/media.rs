use crate::models::*;
use crate::utils::song::test::{SongDate, SongTag};
use crate::DatabasePool;

use diesel::{dsl::exists, ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use futures::stream::{self, StreamExt};
use isolang::Language;
use itertools::Itertools;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug)]
pub struct SongDbInformation {
    pub song_id: Uuid,
    pub album_id: Uuid,
    pub tag: SongTag,
    pub artist_ids: Vec<Uuid>,
    pub album_artist_ids: Vec<Uuid>,
    // Filesystem property
    pub music_folder_id: Uuid,
    pub relative_path: String,
    pub file_hash: u64,
    pub file_size: u64,
}

pub async fn query_all_song_information(pool: &DatabasePool, song_id: Uuid) -> SongDbInformation {
    let song = songs::table
        .filter(songs::id.eq(song_id))
        .select(songs::test::Song::as_select())
        .first(&mut pool.get().await.unwrap())
        .await
        .unwrap();

    let album_id = song.album_id;
    let album_name = albums::table
        .filter(albums::id.eq(song.album_id))
        .select(albums::name)
        .first::<String>(&mut pool.get().await.unwrap())
        .await
        .unwrap();

    let (artist_ids, artist_names): (Vec<Uuid>, Vec<String>) = artists::table
        .inner_join(songs_artists::table)
        .filter(songs_artists::song_id.eq(song_id))
        .select((artists::id, artists::name))
        .get_results::<(Uuid, String)>(&mut pool.get().await.unwrap())
        .await
        .unwrap()
        .into_iter()
        .unzip();
    let artist_ids = artist_ids.into_iter().sorted().collect_vec();
    let artist_names = artist_names.into_iter().sorted().collect_vec();

    let (album_artist_ids, album_artist_names): (Vec<Uuid>, Vec<String>) =
        songs_album_artists::table
            .inner_join(artists::table)
            .inner_join(songs::table)
            .filter(songs::album_id.eq(album_id))
            .select((artists::id, artists::name))
            .get_results::<(Uuid, String)>(&mut pool.get().await.unwrap())
            .await
            .unwrap()
            .into_iter()
            .unzip();
    let album_artist_ids = album_artist_ids.into_iter().sorted().collect_vec();
    let album_artist_names = album_artist_names.into_iter().sorted().collect_vec();

    let tag = SongTag {
        title: song.title,
        album: album_name,
        artists: artist_names,
        album_artists: album_artist_names,
        track_number: song.track_number.map(|i| i as u32),
        track_total: song.track_total.map(|i| i as u32),
        disc_number: song.disc_number.map(|i| i as u32),
        disc_total: song.disc_total.map(|i| i as u32),
        date: SongDate::from_ymd(song.year, song.month, song.day),
        release_date: SongDate::from_ymd(song.release_year, song.release_month, song.release_day),
        original_release_date: SongDate::from_ymd(
            song.original_release_year,
            song.original_release_month,
            song.original_release_day,
        ),
        languages: song
            .languages
            .into_iter()
            .map(|language| Language::from_str(&language.unwrap()).unwrap())
            .collect_vec(),
    };

    SongDbInformation {
        song_id,
        album_id,
        tag,
        artist_ids,
        album_artist_ids,
        music_folder_id: song.music_folder_id,
        relative_path: song.relative_path,
        file_hash: song.file_hash as u64,
        file_size: song.file_size as u64,
    }
}

pub async fn query_all_songs_information(
    pool: &DatabasePool,
) -> HashMap<(Uuid, PathBuf), SongDbInformation> {
    let song_ids = songs::table
        .select(songs::id)
        .get_results(&mut pool.get().await.unwrap())
        .await
        .unwrap();
    stream::iter(song_ids)
        .then(|song_id| async move {
            let result = query_all_song_information(pool, song_id).await;
            (
                (result.music_folder_id, PathBuf::from(&result.relative_path)),
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
            .flat_map(|song_tag| song_tag.album_artists_or_default())
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
    let mut song_db_infos = query_all_songs_information(pool).await;

    for (song_key, song_tag) in song_fs_info {
        let song_db_info = song_db_infos.remove(song_key).unwrap();
        assert_eq!(song_tag.title, song_db_info.tag.title);
        assert_eq!(song_tag.album, song_db_info.tag.album);
        assert_eq!(song_tag.artists, song_db_info.tag.artists);
        assert_eq!(
            song_tag.album_artists_or_default(),
            &song_db_info.tag.album_artists
        );

        assert_eq!(song_tag.track_number, song_db_info.tag.track_number);
        assert_eq!(song_tag.track_total, song_db_info.tag.track_total);
        assert_eq!(song_tag.disc_number, song_db_info.tag.disc_number);
        assert_eq!(song_tag.disc_total, song_db_info.tag.disc_total);

        assert_eq!(song_tag.date_or_default(), song_db_info.tag.date);
        assert_eq!(
            song_tag.release_date_or_default(),
            song_db_info.tag.release_date
        );
        assert_eq!(
            song_tag.original_release_date,
            song_db_info.tag.original_release_date
        );

        assert_eq!(song_tag.languages, song_db_info.tag.languages);
    }
    assert!(song_db_infos.is_empty());
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
                .filter(songs::relative_path.eq(path.to_str().unwrap()))
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

pub async fn song_paths_to_artist_ids(
    pool: &DatabasePool,
    song_fs_info: &HashMap<(Uuid, PathBuf), SongTag>,
) -> Vec<Uuid> {
    let artist_names = song_fs_info
        .clone()
        .into_iter()
        .flat_map(|(_, tag)| [tag.album_artists, tag.artists].concat())
        .unique()
        .collect_vec();

    artists::table
        .select(artists::id)
        .filter(artists::name.eq_any(&artist_names))
        .get_results::<Uuid>(&mut pool.get().await.unwrap())
        .await
        .unwrap()
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
                .filter(songs::relative_path.eq(path.to_str().unwrap()))
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
