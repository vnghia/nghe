use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::slice::SliceIndex;
use std::str::FromStr;

use axum::extract::State;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use fake::{Fake, Faker};
use futures::stream::{self, StreamExt};
use isolang::Language;
use itertools::Itertools;
use uuid::Uuid;

use super::db::SongDbInformation;
use super::fs::SongFsInformation;
use super::{random, TemporaryDb, TemporaryFs};
use crate::config::{ArtConfig, ArtistIndexConfig, ScanConfig};
use crate::database::EncryptionKey;
use crate::models::*;
use crate::open_subsonic::browsing::refresh_music_folders;
use crate::open_subsonic::permission::set_permission;
use crate::open_subsonic::scan::{start_scan, ScanMode, ScanStatistic};
use crate::open_subsonic::test::CommonParams;
use crate::utils::song::file_type::to_extensions;
use crate::utils::song::test::{SongDate, SongTag};
use crate::{Database, DatabasePool};

pub struct Infra {
    pub db: TemporaryDb,
    pub fs: TemporaryFs,
    pub users: Vec<users::User>,
    pub music_folders: Vec<music_folders::MusicFolder>,
    pub song_fs_infos_vec: Vec<Vec<SongFsInformation>>,
}

impl Infra {
    pub async fn new() -> Self {
        let db = TemporaryDb::new_from_env().await;
        let fs = TemporaryFs::default();
        Self { db, fs, users: vec![], music_folders: vec![], song_fs_infos_vec: vec![] }
    }

    pub async fn add_user(mut self, role: Option<users::Role>) -> Self {
        self.users.push(users::User::fake(role).create(self.database()).await);
        let user_index = self.users.len() - 1;
        if !self.music_folders.is_empty() {
            self.permissions(user_index..=user_index, .., true).await;
        }
        self
    }

    pub async fn n_folder(mut self, n_folder: usize) -> Self {
        if !self.music_folders.is_empty() {
            panic!("n_folder should be called only once")
        } else {
            let music_folder_paths =
                (0..n_folder).map(|_| self.fs.create_dir(Faker.fake::<String>())).collect_vec();
            let (upserted_folders, _) =
                refresh_music_folders(self.pool(), &music_folder_paths, &[]).await;
            self.music_folders = upserted_folders;
            self.song_fs_infos_vec = vec![vec![]; n_folder];
            self
        }
    }

    pub async fn permissions<SU, SM>(
        &self,
        user_slice: SU,
        music_folder_slice: SM,
        allow: bool,
    ) -> &Self
    where
        SU: SliceIndex<[users::User], Output = [users::User]>,
        SM: SliceIndex<[music_folders::MusicFolder], Output = [music_folders::MusicFolder]>,
    {
        set_permission(
            self.pool(),
            &self.user_ids(user_slice),
            &self.music_folder_ids(music_folder_slice),
            allow,
        )
        .await
        .unwrap();
        self
    }

    pub async fn only_permissions<SU, SM>(
        &self,
        user_slice: SU,
        music_folder_slice: SM,
        allow: bool,
    ) -> &Self
    where
        SU: SliceIndex<[users::User], Output = [users::User]> + Clone,
        SM: SliceIndex<[music_folders::MusicFolder], Output = [music_folders::MusicFolder]>,
    {
        self.permissions(user_slice.clone(), .., !allow)
            .await
            .permissions(user_slice, music_folder_slice, allow)
            .await
    }

    pub async fn scan<S>(&self, slice: S, scan_mode: Option<ScanMode>) -> ScanStatistic
    where
        S: SliceIndex<[music_folders::MusicFolder], Output = [music_folders::MusicFolder]>,
    {
        start_scan(
            self.pool(),
            scan_mode.unwrap_or(ScanMode::Full),
            self.music_folders[slice].as_ref(),
            &ArtistIndexConfig::default(),
            &self.fs.parsing_config,
            &ScanConfig::default(),
            &ArtConfig::default(),
        )
        .await
        .unwrap()
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

        std::fs::copy(
            music_folder_path.join(old_song_path),
            music_folder_path.join(&new_song_path),
        )
        .unwrap();

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
        S: SliceIndex<[users::User], Output = [users::User]>,
    {
        self.users[slice].as_ref().iter().map(|u| u.id).sorted().collect_vec()
    }

    pub fn to_common_params(&self, index: usize) -> CommonParams {
        self.users[index].to_common_params(self.key())
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
                    .filter(songs::file_size.eq(song_fs_info.file_size as i64))
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

    pub async fn artist_ids<S>(&self, slice: S) -> Vec<Uuid>
    where
        S: SliceIndex<[Vec<SongFsInformation>], Output = [Vec<SongFsInformation>]>,
    {
        let artist_names = self
            .song_fs_infos(slice)
            .iter()
            .flat_map(|s| [s.tag.album_artists.clone(), s.tag.artists.clone()].concat())
            .unique()
            .collect_vec();
        artists::table
            .select(artists::id)
            .filter(artists::name.eq_any(&artist_names))
            .get_results::<Uuid>(&mut self.pool().get().await.unwrap())
            .await
            .unwrap()
            .into_iter()
            .sorted()
            .collect_vec()
    }

    pub async fn album_ids<S>(&self, slice: S) -> Vec<Uuid>
    where
        S: SliceIndex<[Vec<SongFsInformation>], Output = [Vec<SongFsInformation>]>,
    {
        stream::iter(self.song_fs_infos(slice))
            .then(|song_fs_info| async move {
                songs::table
                    .select(songs::album_id)
                    .inner_join(music_folders::table)
                    .filter(
                        music_folders::path.eq(&song_fs_info.music_folder_path.to_str().unwrap()),
                    )
                    .filter(songs::file_hash.eq(song_fs_info.file_hash as i64))
                    .filter(songs::file_size.eq(song_fs_info.file_size as i64))
                    .get_result::<Uuid>(&mut self.pool().get().await.unwrap())
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

    pub async fn song_db_info(&self, song_id: Uuid) -> SongDbInformation {
        let song = songs::table
            .inner_join(music_folders::table)
            .filter(songs::id.eq(song_id))
            .select(songs::test::Song::as_select())
            .first(&mut self.pool().get().await.unwrap())
            .await
            .unwrap();

        let album_id = song.album_id;
        let album_name = albums::table
            .filter(albums::id.eq(song.album_id))
            .select(albums::name)
            .first::<String>(&mut self.pool().get().await.unwrap())
            .await
            .unwrap();

        let (artist_ids, artist_names): (Vec<Uuid>, Vec<String>) = artists::table
            .inner_join(songs_artists::table)
            .filter(songs_artists::song_id.eq(song_id))
            .select((artists::id, artists::name))
            .get_results::<(Uuid, String)>(&mut self.pool().get().await.unwrap())
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
                .get_results::<(Uuid, String)>(&mut self.pool().get().await.unwrap())
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
            track_number: song.track_number.map(|i| i as _),
            track_total: song.track_total.map(|i| i as _),
            disc_number: song.disc_number.map(|i| i as _),
            disc_total: song.disc_total.map(|i| i as _),
            date: SongDate::from_id3_db(song.date),
            release_date: SongDate::from_id3_db(song.release_date),
            original_release_date: SongDate::from_id3_db(song.original_release_date),
            languages: song
                .languages
                .into_iter()
                .map(|language| Language::from_str(&language.unwrap()).unwrap())
                .collect_vec(),
        };

        SongDbInformation {
            tag,
            song_id,
            album_id,
            artist_ids,
            album_artist_ids,
            music_folder: song.music_folder,
            relative_path: song.relative_path,
            file_hash: song.file_hash as u64,
            file_size: song.file_size as u64,
        }
    }

    pub async fn song_db_infos(&self) -> HashMap<(Uuid, u64, u64), SongDbInformation> {
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

            let song_fs_paths = song_fs_infos
                .iter()
                .map(|song_fs_info| song_fs_info.relative_path.as_str())
                .collect::<HashSet<_>>();
            assert!(song_fs_paths.contains(song_db_info.relative_path.as_str()));
        }
    }
}
