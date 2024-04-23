use std::collections::{HashMap, HashSet};
use std::ops::Add;
use std::path::Path;
use std::slice::SliceIndex;
use std::str::FromStr;

use axum::extract::State;
use diesel::dsl::{Filter, IsNotNull};
use diesel::{
    ExpressionMethods, OptionalExtension, PgExpressionMethods, QueryDsl, SelectableHelper,
};
use diesel_async::RunQueryDsl;
use fake::{Fake, Faker};
use futures::stream::{self, StreamExt};
use isolang::Language;
use itertools::Itertools;
use nghe_types::params::CommonParams;
use nghe_types::scan::start_scan::{ScanMode, StartScanParams};
use uuid::Uuid;
use xxhash_rust::xxh3::xxh3_64;

use super::db::SongDbInformation;
use super::fs::SongFsInformation;
use super::{picture, random, TemporaryDb, TemporaryFs, User};
use crate::config::{ArtistIndexConfig, ScanConfig};
use crate::database::EncryptionKey;
use crate::models::*;
use crate::open_subsonic::music_folder::test::add_music_folder;
use crate::open_subsonic::permission::{add_permission, remove_permission};
use crate::open_subsonic::scan::test::initialize_scan;
use crate::open_subsonic::scan::{start_scan, ScanStat};
use crate::open_subsonic::test::id3::*;
use crate::utils::song::file_type::{picture_to_extension, to_extensions};
use crate::utils::song::test::SongTag;
use crate::utils::song::MediaDateMbz;
use crate::{Database, DatabasePool};

pub struct Infra {
    pub db: TemporaryDb,
    pub fs: TemporaryFs,
    pub users: Vec<User>,
    pub music_folders: Vec<music_folders::MusicFolder>,
    pub song_fs_infos_vec: Vec<Vec<SongFsInformation>>,
}

impl Infra {
    pub async fn new() -> Self {
        let db = TemporaryDb::new_from_env().await;
        let fs = TemporaryFs::default();
        Self { db, fs, users: vec![], music_folders: vec![], song_fs_infos_vec: vec![] }
    }

    pub async fn add_user(self, role: Option<users::Role>) -> Self {
        self.add_user_allow(role, true).await
    }

    pub async fn add_user_allow(mut self, role: Option<users::Role>, allow: bool) -> Self {
        self.users.push(User::fake(role).create(self.database(), allow).await);
        self
    }

    pub async fn add_folder(mut self, allow: bool) -> Self {
        let pathbuf = self.fs.create_dir(Faker.fake::<String>()).canonicalize().unwrap();

        let name = pathbuf.file_stem().unwrap().to_os_string().into_string().unwrap();
        let path = pathbuf.into_os_string().into_string().unwrap();
        let id = add_music_folder(self.pool(), &name, &path, allow).await.unwrap();

        self.music_folders.push(music_folders::MusicFolder { id, name, path });
        self.song_fs_infos_vec.push(vec![]);

        self
    }

    pub async fn n_folder(mut self, n_folder: usize) -> Self {
        for _ in 0..n_folder {
            self = self.add_folder(true).await;
        }

        self
    }

    pub async fn add_permission<U: Into<Option<usize>>, M: Into<Option<usize>>>(
        &self,
        user_idx: U,
        music_folder_idx: M,
    ) -> &Self {
        add_permission(
            self.pool(),
            user_idx.into().map(|i| self.user_id(i)),
            music_folder_idx.into().map(|i| self.music_folder_id(i)),
        )
        .await
        .unwrap();
        self
    }

    pub async fn add_permissions<SU, SM>(&self, user_slice: SU, music_folder_slice: SM) -> &Self
    where
        SU: SliceIndex<[User], Output = [User]>,
        SM: SliceIndex<[music_folders::MusicFolder], Output = [music_folders::MusicFolder]>,
    {
        for (user_id, music_folder_id) in self
            .user_ids(user_slice)
            .into_iter()
            .cartesian_product(self.music_folder_ids(music_folder_slice))
        {
            add_permission(self.pool(), Some(user_id), Some(music_folder_id)).await.unwrap();
        }
        self
    }

    pub async fn remove_permission<U: Into<Option<usize>>, M: Into<Option<usize>>>(
        &self,
        user_idx: U,
        music_folder_idx: M,
    ) -> &Self {
        remove_permission(
            self.pool(),
            user_idx.into().map(|i| self.user_id(i)),
            music_folder_idx.into().map(|i| self.music_folder_id(i)),
        )
        .await
        .unwrap();
        self
    }

    pub async fn remove_permissions<SU, SM>(&self, user_slice: SU, music_folder_slice: SM) -> &Self
    where
        SU: SliceIndex<[User], Output = [User]>,
        SM: SliceIndex<[music_folders::MusicFolder], Output = [music_folders::MusicFolder]>,
    {
        for (user_id, music_folder_id) in self
            .user_ids(user_slice)
            .into_iter()
            .cartesian_product(self.music_folder_ids(music_folder_slice))
        {
            remove_permission(self.pool(), Some(user_id), Some(music_folder_id)).await.unwrap();
        }
        self
    }

    pub async fn scan<S>(&self, slice: S, scan_mode: Option<ScanMode>) -> ScanStat
    where
        S: SliceIndex<[music_folders::MusicFolder], Output = [music_folders::MusicFolder]>,
    {
        let result = stream::iter(self.music_folder_ids(slice))
            .then(move |id| async move {
                let scan_started_at = initialize_scan(self.pool(), id).await.unwrap();
                // Postgres timestamp resolution is microsecond.
                // So we wait for a moment to make sure that there is no overlap scans.
                if cfg!(target_os = "freebsd") {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
                start_scan(
                    self.pool(),
                    scan_started_at,
                    StartScanParams { id, mode: scan_mode.unwrap_or(ScanMode::Full) },
                    &ArtistIndexConfig::default(),
                    &self.fs.parsing_config,
                    &ScanConfig::default(),
                    &self.fs.art_config,
                )
                .await
                .unwrap()
            })
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .reduce(ScanStat::add)
            .unwrap();
        if cfg!(target_os = "freebsd") {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
        result
    }

    pub fn add_n_song(&mut self, index: usize, n_song: usize) -> &mut Self {
        self.add_songs(index, fake::vec![SongTag; n_song])
    }

    pub fn add_songs(&mut self, index: usize, song_tags: Vec<SongTag>) -> &mut Self {
        self.song_fs_infos_vec[index].extend(self.fs.create_random_paths_media_files(
            &self.music_folders[index].path,
            song_tags,
            &to_extensions(),
        ));
        self
    }

    pub fn delete_songs(&mut self, index: usize, delete_mask: &[bool]) -> &mut Self {
        self.song_fs_infos_vec[index] = delete_mask
            .iter()
            .copied()
            .zip(std::mem::take(&mut self.song_fs_infos_vec[index]))
            .filter_map(|(d, s)| {
                if d {
                    std::fs::remove_file(s.absolute_path()).unwrap();
                    if s.lrc.is_some() {
                        std::fs::remove_file(s.absolute_path().with_extension("lrc")).unwrap();
                    }
                    None
                } else {
                    Some(s)
                }
            })
            .collect();
        self
    }

    pub fn delete_song(&mut self, music_folder_index: usize, song_index: usize) -> &mut Self {
        let mut delete_mask = vec![false; self.song_fs_infos_vec[music_folder_index].len()];
        delete_mask[song_index] = true;
        self.delete_songs(music_folder_index, &delete_mask)
    }

    pub fn delete_n_song(&mut self, index: usize, n_song: usize) -> &mut Self {
        self.delete_songs(
            index,
            &random::gen_bool_mask(self.song_fs_infos_vec[index].len(), n_song),
        )
    }

    pub fn update_songs(
        &mut self,
        index: usize,
        update_mask: &[bool],
        song_tags: Vec<SongTag>,
    ) -> &mut Self {
        let new_song_fs_infos = self.fs.create_media_files(
            &self.music_folders[index].path,
            update_mask
                .iter()
                .copied()
                .zip(self.song_fs_infos_vec[index].iter())
                .filter_map(|(u, s)| if u { Some(s.relative_path.clone()) } else { None })
                .collect(),
            song_tags,
            false,
        );
        update_mask
            .iter()
            .copied()
            .enumerate()
            .filter_map(|(i, u)| if u { Some(i) } else { None })
            .zip(new_song_fs_infos)
            .for_each(|(i, s)| {
                self.song_fs_infos_vec[index][i] = s;
            });
        self
    }

    pub fn update_song(
        &mut self,
        music_folder_index: usize,
        song_index: usize,
        song_tag: SongTag,
    ) -> &mut Self {
        let mut update_mask = vec![false; self.song_fs_infos_vec[music_folder_index].len()];
        update_mask[song_index] = true;
        self.update_songs(music_folder_index, &update_mask, vec![song_tag])
    }

    pub fn update_n_song(&mut self, index: usize, n_song: usize) -> &mut Self {
        self.update_songs(
            index,
            &random::gen_bool_mask(self.song_fs_infos_vec[index].len(), n_song),
            fake::vec![SongTag; n_song],
        )
    }

    pub fn copy_song<P: AsRef<Path>>(
        &mut self,
        music_folder_index: usize,
        src_index: usize,
        dst_path: P,
    ) -> &mut Self {
        let music_folder_path = Path::new(&self.music_folders[music_folder_index].path);

        let old_song_tag = self.song_fs_infos_vec[music_folder_index][src_index].clone();
        let old_song_path = Path::new(&old_song_tag.relative_path);

        let new_song_path = dst_path.as_ref().with_extension(old_song_path.extension().unwrap());
        assert!(!new_song_path.is_absolute());

        let old_abs_song_path = music_folder_path.join(old_song_path);
        let new_abs_song_path = music_folder_path.join(&new_song_path);

        std::fs::copy(&old_abs_song_path, &new_abs_song_path).unwrap();
        if old_song_tag.lrc.is_some() {
            std::fs::copy(
                old_abs_song_path.with_extension("lrc"),
                new_abs_song_path.with_extension("lrc"),
            )
            .unwrap();
        }

        self.song_fs_infos_vec[music_folder_index].push(SongFsInformation {
            relative_path: new_song_path.to_str().unwrap().to_owned(),
            ..old_song_tag
        });
        self
    }

    pub fn database(&self) -> &Database {
        self.db.database()
    }

    pub fn pool(&self) -> &DatabasePool {
        &self.database().pool
    }

    pub fn key(&self) -> &EncryptionKey {
        &self.database().key
    }

    pub fn state(&self) -> State<Database> {
        self.db.state()
    }

    pub fn user_id(&self, index: usize) -> Uuid {
        self.user_ids(index..=index)[0]
    }

    pub fn user_ids<S>(&self, slice: S) -> Vec<Uuid>
    where
        S: SliceIndex<[User], Output = [User]>,
    {
        self.users[slice].as_ref().iter().map(|u| u.id).sorted().collect_vec()
    }

    pub fn to_common_params(&self, index: usize) -> CommonParams {
        (&self.users[index]).into()
    }

    pub fn music_folder_id(&self, index: usize) -> Uuid {
        self.music_folder_ids(index..=index)[0]
    }

    pub fn music_folder_ids<S>(&self, slice: S) -> Vec<Uuid>
    where
        S: SliceIndex<[music_folders::MusicFolder], Output = [music_folders::MusicFolder]>,
    {
        self.music_folders[slice].as_ref().iter().map(|f| f.id).sorted().collect_vec()
    }

    pub fn song_fs_infos<S>(&self, slice: S) -> Vec<SongFsInformation>
    where
        S: SliceIndex<[Vec<SongFsInformation>], Output = [Vec<SongFsInformation>]>,
    {
        self.song_fs_infos_vec[slice].as_ref().iter().flat_map(|v| v.clone()).collect()
    }

    pub async fn song_ids<S>(&self, slice: S) -> Vec<Uuid>
    where
        S: SliceIndex<[Vec<SongFsInformation>], Output = [Vec<SongFsInformation>]>,
    {
        stream::iter(self.song_fs_infos(slice))
            .then(|song_fs_info| async move {
                songs::table
                    .select(songs::id)
                    .inner_join(music_folders::table)
                    .filter(
                        music_folders::path.eq(&song_fs_info.music_folder_path.to_str().unwrap()),
                    )
                    .filter(songs::file_hash.eq(song_fs_info.file_hash as i64))
                    .filter(songs::file_size.eq(song_fs_info.file_size as i32))
                    .get_result::<Uuid>(&mut self.pool().get().await.unwrap())
                    .await
                    .unwrap()
            })
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .sorted()
            .collect()
    }

    pub async fn song_cover_art_ids<S>(&self, slice: S) -> Vec<Uuid>
    where
        S: SliceIndex<[Vec<SongFsInformation>], Output = [Vec<SongFsInformation>]>,
    {
        stream::iter(self.song_fs_infos(slice))
            .then(|song_fs_info| async move {
                let picture = song_fs_info.tag.picture.unwrap();
                let file_format = picture_to_extension(picture.mime_type().unwrap());
                let data = picture.data();
                let file_hash = xxh3_64(data);
                let file_size = data.len();

                song_cover_arts::table
                    .select(song_cover_arts::id)
                    .filter(song_cover_arts::format.eq(file_format))
                    .filter(song_cover_arts::file_hash.eq(file_hash as i64))
                    .filter(song_cover_arts::file_size.eq(file_size as i32))
                    .get_result::<Uuid>(&mut self.pool().get().await.unwrap())
                    .await
                    .unwrap()
            })
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .sorted()
            .collect()
    }

    pub async fn artist_ids(&self, artist_no_ids: &[artists::ArtistNoId]) -> Vec<Uuid> {
        stream::iter(artist_no_ids)
            .then(|artist_no_id| async move {
                if let Some(artist_mbz_id) = artist_no_id.mbz_id {
                    artists::table
                        .select(artists::id)
                        .filter(artists::mbz_id.eq(artist_mbz_id))
                        .get_result::<Uuid>(&mut self.pool().get().await.unwrap())
                        .await
                        .unwrap()
                } else {
                    artists::table
                        .select(artists::id)
                        .filter(artists::name.eq(artist_no_id.name.as_ref()))
                        .get_result::<Uuid>(&mut self.pool().get().await.unwrap())
                        .await
                        .unwrap()
                }
            })
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .sorted()
            .collect()
    }

    pub fn song_artist_no_ids<S>(&self, slice: S) -> Vec<artists::ArtistNoId>
    where
        S: SliceIndex<[Vec<SongFsInformation>], Output = [Vec<SongFsInformation>]>,
    {
        self.song_fs_infos(slice).iter().flat_map(|s| s.tag.artists.clone()).unique().collect_vec()
    }

    pub fn album_artist_no_ids<S>(&self, slice: S) -> Vec<artists::ArtistNoId>
    where
        S: SliceIndex<[Vec<SongFsInformation>], Output = [Vec<SongFsInformation>]>,
    {
        self.song_fs_infos(slice)
            .iter()
            .flat_map(|s| s.tag.album_artists.clone())
            .unique()
            .collect_vec()
    }

    pub fn artist_no_ids<S>(&self, slice: S) -> Vec<artists::ArtistNoId>
    where
        S: SliceIndex<[Vec<SongFsInformation>], Output = [Vec<SongFsInformation>]> + Clone,
    {
        [self.song_artist_no_ids(slice.clone()), self.album_artist_no_ids(slice.clone())]
            .concat()
            .into_iter()
            .unique()
            .collect_vec()
    }

    pub async fn album_ids(&self, album_no_ids: &[albums::AlbumNoId]) -> Vec<Uuid> {
        stream::iter(album_no_ids)
            .then(|album_no_id| async move {
                if let Some(album_mbz_id) = album_no_id.mbz_id {
                    albums::table
                        .select(albums::id)
                        .filter(albums::mbz_id.eq(album_mbz_id))
                        .get_result::<Uuid>(&mut self.pool().get().await.unwrap())
                        .await
                        .unwrap()
                } else {
                    albums::table
                        .select(albums::id)
                        .filter(albums::name.eq(album_no_id.name.as_ref()))
                        .filter(albums::year.is_not_distinct_from(album_no_id.date.year))
                        .filter(albums::month.is_not_distinct_from(album_no_id.date.month))
                        .filter(albums::day.is_not_distinct_from(album_no_id.date.day))
                        .filter(
                            albums::release_year
                                .is_not_distinct_from(album_no_id.release_date.year),
                        )
                        .filter(
                            albums::release_month
                                .is_not_distinct_from(album_no_id.release_date.month),
                        )
                        .filter(
                            albums::release_day.is_not_distinct_from(album_no_id.release_date.day),
                        )
                        .filter(
                            albums::original_release_year
                                .is_not_distinct_from(album_no_id.original_release_date.year),
                        )
                        .filter(
                            albums::original_release_month
                                .is_not_distinct_from(album_no_id.original_release_date.month),
                        )
                        .filter(
                            albums::original_release_day
                                .is_not_distinct_from(album_no_id.original_release_date.day),
                        )
                        .filter(albums::mbz_id.is_null())
                        .get_result::<Uuid>(&mut self.pool().get().await.unwrap())
                        .await
                        .unwrap()
                }
            })
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .unique()
            .sorted()
            .collect_vec()
    }

    pub fn album_no_ids<S>(&self, slice: S) -> Vec<albums::AlbumNoId>
    where
        S: SliceIndex<[Vec<SongFsInformation>], Output = [Vec<SongFsInformation>]>,
    {
        self.song_fs_infos(slice).iter().map(|s| s.tag.album.clone().into()).unique().collect_vec()
    }

    pub fn get_song_artist_db() -> Filter<GetBasicArtistId3Db, IsNotNull<songs_artists::artist_id>>
    {
        // inner join = left join + not null
        get_basic_artist_id3_db().filter(songs_artists::artist_id.is_not_null())
    }

    pub fn get_album_artist_db()
    -> Filter<GetBasicArtistId3Db, IsNotNull<songs_album_artists::album_artist_id>> {
        // inner join = left join + not null
        get_basic_artist_id3_db().filter(songs_album_artists::album_artist_id.is_not_null())
    }

    pub async fn song_db_info(&self, song_id: Uuid) -> SongDbInformation {
        let song = songs::table
            .inner_join(music_folders::table)
            .filter(songs::id.eq(song_id))
            .select(songs::test::Song::as_select())
            .first(&mut self.pool().get().await.unwrap())
            .await
            .unwrap();
        let song_media = MediaDateMbz {
            name: song.title,
            date: song.date.into(),
            release_date: song.release_date.into(),
            original_release_date: song.original_release_date.into(),
            mbz_id: song.mbz_id,
        };

        let album = get_basic_album_id3_db()
            .filter(albums::id.eq(song.album_id))
            .get_result::<BasicAlbumId3Db>(&mut self.pool().get().await.unwrap())
            .await
            .unwrap();
        let album_media = MediaDateMbz {
            name: album.no_id.name.into_owned(),
            date: album.no_id.date.into(),
            release_date: album.no_id.release_date.into(),
            original_release_date: album.no_id.original_release_date.into(),
            mbz_id: album.no_id.mbz_id,
        };

        let (artist_ids, artist_no_ids): (Vec<_>, Vec<_>) = Self::get_song_artist_db()
            .filter(songs::id.eq(song_id))
            .get_results::<BasicArtistId3Db>(&mut self.pool().get().await.unwrap())
            .await
            .unwrap()
            .into_iter()
            .map(|a| (a.id, a.no_id))
            .unzip();
        let artist_ids = artist_ids.into_iter().sorted().collect_vec();
        let artist_no_ids = artist_no_ids.into_iter().sorted().collect_vec();

        let (album_artist_ids, album_artist_no_ids): (Vec<_>, Vec<_>) = Self::get_album_artist_db()
            .filter(songs::id.eq(song_id))
            .get_results::<BasicArtistId3Db>(&mut self.pool().get().await.unwrap())
            .await
            .unwrap()
            .into_iter()
            .map(|a| (a.id, a.no_id))
            .unzip();
        let album_artist_ids = album_artist_ids.into_iter().sorted().collect_vec();
        let album_artist_no_ids = album_artist_no_ids.into_iter().sorted().collect_vec();

        let genres = get_basic_genre_id3_db()
            .inner_join(songs::table)
            .filter(songs::id.eq(song_id))
            .select(BasicGenreId3Db::as_select())
            .get_results::<BasicGenreId3Db>(&mut self.pool().get().await.unwrap())
            .await
            .unwrap();

        let picture = picture::from_id(self.pool(), song.cover_art_id, &self.fs.art_config).await;

        let tag = SongTag {
            song: song_media,
            album: album_media,
            artists: artist_no_ids,
            album_artists: album_artist_no_ids,
            track_number: song.track_number.map(|i| i as _),
            track_total: song.track_total.map(|i| i as _),
            disc_number: song.disc_number.map(|i| i as _),
            disc_total: song.disc_total.map(|i| i as _),
            languages: song
                .languages
                .into_iter()
                .map(|language| Language::from_str(&language.unwrap()).unwrap())
                .collect_vec(),
            genres,
            picture,
        };

        let lrc = get_lyric_id3_db()
            .filter(songs::id.eq(song_id))
            .filter(lyrics::external)
            .select((LyricId3Db::as_select(), lyrics::description, lyrics::external))
            .get_result::<(LyricId3Db, String, bool)>(&mut self.pool().get().await.unwrap())
            .await
            .optional()
            .unwrap()
            .map(|(l, d, e)| (l.into(), d, e).into());

        SongDbInformation {
            tag,
            lrc,
            song_id,
            album_id: album.id,
            artist_ids,
            album_artist_ids,
            music_folder: song.music_folder,
            relative_path: song.relative_path,
            file_hash: song.file_hash as _,
            file_size: song.file_size as _,
        }
    }

    pub async fn song_db_infos(&self) -> HashMap<(Uuid, u64, u32), SongDbInformation> {
        let song_ids = songs::table
            .select(songs::id)
            .get_results(&mut self.pool().get().await.unwrap())
            .await
            .unwrap();
        stream::iter(song_ids)
            .then(|song_id| async move {
                let result = self.song_db_info(song_id).await;
                ((result.music_folder.id, result.file_hash, result.file_size), result)
            })
            .collect::<HashMap<_, _>>()
            .await
    }

    pub async fn assert_song_infos(&self) {
        let music_folders =
            self.music_folders.iter().map(|f| (f.path.as_str(), f.id)).collect::<HashMap<_, _>>();
        let song_fs_infos_map =
            self.song_fs_infos(..).into_iter().into_group_map_by(|song_fs_info| {
                (
                    music_folders[song_fs_info
                        .music_folder_path
                        .to_str()
                        .expect("non utf-8 path encountered")],
                    song_fs_info.file_hash,
                    song_fs_info.file_size,
                )
            });
        let mut song_db_infos = self.song_db_infos().await;
        assert_eq!(song_fs_infos_map.len(), song_db_infos.len());

        for (song_key_info, song_fs_infos) in song_fs_infos_map {
            let song_db_info = song_db_infos.remove(&song_key_info).unwrap();
            let song_fs_info = &song_fs_infos[0];
            let song_fs_tag = &song_fs_info.tag;
            let song_db_tag = &song_db_info.tag;

            let song_fs = &song_fs_tag.song;
            let song_db = &song_db_tag.song;
            assert_eq!(song_fs.name, song_db.name);
            assert_eq!(
                song_fs.date_or_default().or(song_fs_tag.album.date_or_default()),
                song_db.date
            );
            assert_eq!(song_fs.release_date_or_default(), song_db.release_date);
            assert_eq!(song_fs.original_release_date, song_db.original_release_date);
            assert_eq!(song_fs.mbz_id, song_db.mbz_id);

            let album_fs = &song_fs_tag.album;
            let album_db = &song_db_tag.album;
            assert_eq!(album_fs.name, album_db.name);
            assert_eq!(album_fs.date_or_default(), album_db.date);
            assert_eq!(album_fs.release_date_or_default(), album_db.release_date);
            assert_eq!(album_fs.original_release_date, album_db.original_release_date);
            assert_eq!(album_fs.mbz_id, album_db.mbz_id);

            assert_eq!(song_fs_tag.artists, song_db_tag.artists);
            assert_eq!(song_fs_tag.album_artists_or_default(), &song_db_tag.album_artists);

            assert_eq!(song_fs_tag.track_number, song_db_tag.track_number);
            assert_eq!(song_fs_tag.track_total, song_db_tag.track_total);
            assert_eq!(song_fs_tag.disc_number, song_db_tag.disc_number);
            assert_eq!(song_fs_tag.disc_total, song_db_tag.disc_total);

            assert_eq!(song_fs_tag.languages, song_db_tag.languages);

            assert_eq!(song_fs_tag.picture, song_db_tag.picture);

            assert_eq!(song_fs_info.file_hash, song_db_info.file_hash);
            assert_eq!(song_fs_info.file_size, song_db_info.file_size);

            assert_eq!(song_fs_info.lrc, song_db_info.lrc);

            let song_fs_paths = song_fs_infos
                .iter()
                .map(|song_fs_info| song_fs_info.relative_path.as_str())
                .collect::<HashSet<_>>();
            assert!(song_fs_paths.contains(song_db_info.relative_path.as_str()));
        }
    }

    pub async fn assert_artist_infos<S>(&self, slice: S)
    where
        S: SliceIndex<[Vec<SongFsInformation>], Output = [Vec<SongFsInformation>]> + Clone,
    {
        self.assert_artist_no_ids(&self.artist_no_ids(slice)).await;
    }

    pub async fn assert_artist_no_ids(&self, artist_no_ids: &[artists::ArtistNoId]) {
        assert_eq!(
            self.artist_ids(artist_no_ids).await,
            artists::table
                .select(artists::id)
                .get_results::<Uuid>(&mut self.pool().get().await.unwrap())
                .await
                .unwrap()
                .into_iter()
                .sorted()
                .collect_vec(),
        );
    }

    pub async fn assert_song_artist_infos<S>(&self, slice: S)
    where
        S: SliceIndex<[Vec<SongFsInformation>], Output = [Vec<SongFsInformation>]>,
    {
        self.assert_song_artist_no_ids(&self.song_artist_no_ids(slice)).await;
    }

    pub async fn assert_song_artist_no_ids(&self, artist_no_ids: &[artists::ArtistNoId]) {
        assert_eq!(
            self.artist_ids(artist_no_ids).await,
            Self::get_song_artist_db()
                .select(artists::id)
                .get_results::<Uuid>(&mut self.pool().get().await.unwrap())
                .await
                .unwrap()
                .into_iter()
                .sorted()
                .collect_vec(),
        );
    }

    pub async fn assert_album_artist_infos<S>(&self, slice: S)
    where
        S: SliceIndex<[Vec<SongFsInformation>], Output = [Vec<SongFsInformation>]>,
    {
        self.assert_album_artist_no_ids(&self.album_artist_no_ids(slice)).await;
    }

    pub async fn assert_album_artist_no_ids(&self, artist_no_ids: &[artists::ArtistNoId]) {
        assert_eq!(
            self.artist_ids(artist_no_ids).await,
            Self::get_album_artist_db()
                .select(artists::id)
                .get_results::<Uuid>(&mut self.pool().get().await.unwrap())
                .await
                .unwrap()
                .into_iter()
                .sorted()
                .collect_vec(),
        );
    }

    pub async fn assert_album_infos(&self, album_no_ids: &[albums::AlbumNoId]) {
        assert_eq!(
            self.album_ids(album_no_ids).await,
            albums::table
                .select(albums::id)
                .load::<Uuid>(&mut self.pool().get().await.unwrap())
                .await
                .unwrap()
                .into_iter()
                .sorted()
                .collect_vec(),
        );
    }
}
