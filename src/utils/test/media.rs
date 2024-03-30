use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

use diesel::dsl::exists;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use futures::stream::{self, StreamExt};
use isolang::Language;
use itertools::Itertools;
use uuid::Uuid;

use super::fs::SongFsInformation;
use crate::models::*;
use crate::utils::song::test::{SongDate, SongTag};
use crate::DatabasePool;

#[derive(Debug)]
pub struct SongDbInformation {
    pub song_id: Uuid,
    pub album_id: Uuid,
    pub tag: SongTag,
    pub artist_ids: Vec<Uuid>,
    pub album_artist_ids: Vec<Uuid>,
    // Filesystem property
    pub music_folder_id: Uuid,
    pub music_folder_path: PathBuf,
    pub relative_path: String,
    pub file_hash: u64,
    pub file_size: u64,
}

pub async fn query_all_song_information(pool: &DatabasePool, song_id: Uuid) -> SongDbInformation {
    let (song, music_folder_path): (_, String) = songs::table
        .inner_join(music_folders::table)
        .filter(songs::id.eq(song_id))
        .select((songs::test::Song::as_select(), music_folders::path))
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
            .filter(songs::id.eq(song_id))
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
        music_folder_path: PathBuf::from(music_folder_path),
        relative_path: song.relative_path,
        file_hash: song.file_hash as u64,
        file_size: song.file_size as u64,
    }
}

pub async fn query_all_songs_information(
    pool: &DatabasePool,
) -> HashMap<(PathBuf, String), SongDbInformation> {
    let song_ids =
        songs::table.select(songs::id).get_results(&mut pool.get().await.unwrap()).await.unwrap();
    stream::iter(song_ids)
        .then(|song_id| async move {
            let result = query_all_song_information(pool, song_id).await;
            ((result.music_folder_path.clone(), result.relative_path.clone()), result)
        })
        .collect::<HashMap<_, _>>()
        .await
}

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

pub async fn assert_songs_info(pool: &DatabasePool, song_fs_infos: &[SongFsInformation]) {
    let mut song_db_infos = query_all_songs_information(pool).await;
    assert_eq!(song_fs_infos.len(), song_db_infos.len());

    for song_fs_info in song_fs_infos {
        let song_db_info = song_db_infos
            .remove(&(song_fs_info.music_folder_path.clone(), song_fs_info.relative_path.clone()))
            .unwrap();
        let song_fs_tag = &song_fs_info.tag;
        let song_db_tag = &song_db_info.tag;

        assert_eq!(song_fs_tag.title, song_db_tag.title);
        assert_eq!(song_fs_tag.album, song_db_tag.album);
        assert_eq!(song_fs_tag.artists, song_db_tag.artists);
        assert_eq!(song_fs_tag.album_artists_or_default(), &song_db_tag.album_artists,);

        assert_eq!(song_fs_tag.track_number, song_db_tag.track_number);
        assert_eq!(song_fs_tag.track_total, song_db_tag.track_total);
        assert_eq!(song_fs_tag.disc_number, song_db_tag.disc_number);
        assert_eq!(song_fs_tag.disc_total, song_db_tag.disc_total);

        assert_eq!(song_fs_tag.date_or_default(), song_db_tag.date);
        assert_eq!(song_fs_tag.release_date_or_default(), song_db_tag.release_date);
        assert_eq!(song_fs_tag.original_release_date, song_db_tag.original_release_date);

        assert_eq!(song_fs_tag.languages, song_db_tag.languages);

        assert_eq!(song_fs_info.file_hash, song_db_info.file_hash);
        assert_eq!(song_fs_info.file_size, song_db_info.file_size);
    }
    assert!(song_db_infos.is_empty());
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
