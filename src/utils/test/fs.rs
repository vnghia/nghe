mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

use super::asset::get_media_asset_path;
use crate::config::parsing::ParsingConfig;
use crate::models::*;
use crate::utils::song::file_type::to_extension;
use crate::utils::song::file_type::SONG_FILE_TYPES;
use crate::utils::song::test::SongTag;
use crate::utils::song::SongInformation;
use crate::{open_subsonic::browsing::refresh_music_folders, DatabasePool};

use concat_string::concat_string;
use fake::{Fake, Faker};
use itertools::Itertools;
use lofty::{FileType, TagExt, TagType, TaggedFileExt};
use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::{fs::*, io::Write};
use tempfile::{Builder, TempDir};
use uuid::Uuid;

pub struct TemporaryFs {
    root: TempDir,
    pub parsing_config: ParsingConfig,
}

#[allow(clippy::new_without_default)]
impl TemporaryFs {
    pub fn new() -> Self {
        Self {
            root: Builder::new()
                .prefix(built_info::PKG_NAME)
                .tempdir()
                .expect("can not create temporary directory"),
            parsing_config: Default::default(),
        }
    }

    pub fn join_root_path<PR: AsRef<Path>, P: AsRef<Path>>(root_path: PR, path: P) -> PathBuf {
        if path.as_ref().is_absolute() {
            if !path.as_ref().starts_with(root_path) {
                panic!("path is not a children of root temp directory");
            } else {
                path.as_ref().into()
            }
        } else {
            root_path.as_ref().join(path)
        }
    }

    fn get_absolute_path<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        Self::join_root_path(self.get_root_path(), path)
    }

    fn create_parent_dir<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        let path = self.get_absolute_path(path);
        self.create_dir(path.parent().unwrap());
        path
    }

    pub fn get_root_path(&self) -> &Path {
        self.root.path()
    }

    pub fn create_dir<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        let path = self.get_absolute_path(path);
        create_dir_all(&path).expect("can not create temporary dir");
        path
    }

    pub fn create_file<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        let path = self.create_parent_dir(path);

        File::create(&path)
            .expect("can not open temporary file")
            .write_all(Faker.fake::<String>().as_bytes())
            .expect("can not write to temporary file");
        path
    }

    pub fn create_media_file<PM: AsRef<Path>, P: AsRef<Path>>(
        &self,
        music_folder_path: PM,
        path: P,
        song_tag: SongTag,
    ) -> PathBuf {
        let path = self.create_parent_dir(Self::join_root_path(music_folder_path, path));
        let file_type = FileType::from_path(&path).unwrap();

        std::fs::copy(get_media_asset_path(&file_type), &path)
            .expect("can not copy original media file to temp directory");

        let tag_type = lofty::read_from_path(&path)
            .expect("can not read original media file")
            .primary_tag_type();

        match tag_type {
            TagType::Id3v2 => {
                song_tag
                    .into_id3v2(&self.parsing_config.id3v2)
                    .save_to_path(&path)
                    .expect("can not write tag to media file");
            }
            TagType::VorbisComments => {
                song_tag
                    .into_vorbis_comments(&self.parsing_config.vorbis)
                    .save_to_path(&path)
                    .expect("can not write tag to media file");
            }
            _ => unreachable!("media tag type not supported"),
        };

        path
    }

    pub fn create_media_files<PM: AsRef<Path>, P: AsRef<Path>>(
        &self,
        music_folder_id: Uuid,
        music_folder_path: &PM,
        paths: &[P],
        song_tags: Vec<SongTag>,
    ) -> HashMap<(Uuid, PathBuf), SongTag> {
        paths
            .iter()
            .zip(song_tags)
            .map(|(path, song_tag)| {
                (
                    (
                        music_folder_id,
                        self.create_media_file(music_folder_path, path, song_tag.clone())
                            .strip_prefix(music_folder_path)
                            .unwrap()
                            .to_path_buf(),
                    ),
                    song_tag,
                )
            })
            .collect::<HashMap<_, _>>()
    }

    pub fn create_random_paths<PR: AsRef<Path>, OS: AsRef<OsStr>>(
        &self,
        root_path: PR,
        n_path: usize,
        max_depth: usize,
        extensions: &[OS],
    ) -> Vec<PathBuf> {
        let root_path = self.get_absolute_path(root_path);
        (0..n_path)
            .map(|_| {
                let ext = extensions.choose(&mut rand::thread_rng()).unwrap();
                Self::join_root_path(
                    &root_path,
                    PathBuf::from(
                        fake::vec![String; 1..(max_depth + 1)].join(std::path::MAIN_SEPARATOR_STR),
                    )
                    .with_extension(ext),
                )
            })
            .collect_vec()
    }

    pub fn create_random_paths_media_files<PM: AsRef<Path>, OS: AsRef<OsStr>>(
        &self,
        music_folder_id: Uuid,
        music_folder_path: &PM,
        song_tags: Vec<SongTag>,
        extensions: &[OS],
    ) -> HashMap<(Uuid, PathBuf), SongTag> {
        let n_song = song_tags.len();
        self.create_media_files(
            music_folder_id,
            music_folder_path,
            &self.create_random_paths(music_folder_path, n_song, 3, extensions),
            song_tags,
        )
    }

    pub fn join_paths<P: AsRef<Path>>(&self, paths: &[P]) -> Vec<PathBuf> {
        paths
            .iter()
            .map(|path| self.get_absolute_path(path))
            .collect()
    }

    pub fn canonicalize_paths<P: AsRef<Path>>(&self, paths: &[P]) -> Vec<PathBuf> {
        paths
            .iter()
            .map(std::fs::canonicalize)
            .collect::<Result<Vec<_>, _>>()
            .expect("can not canonicalize temp path")
    }

    pub async fn create_music_folders(
        &self,
        pool: &DatabasePool,
        n_folder: usize,
    ) -> Vec<music_folders::MusicFolder> {
        let music_folder_paths = (0..n_folder)
            .map(|_| self.create_dir(Faker.fake::<String>()))
            .collect_vec();
        let (upserted_folders, _) = refresh_music_folders(pool, &music_folder_paths, &[]).await;
        upserted_folders
    }
}

#[test]
fn test_roundtrip_media_file() {
    let fs = TemporaryFs::new();

    for file_type in SONG_FILE_TYPES {
        let song_tag = Faker.fake::<SongTag>();
        let path = fs.create_media_file(
            fs.get_root_path(),
            concat_string!("test.", to_extension(&file_type)),
            song_tag.clone(),
        );
        let read_song_tag = SongInformation::read_from(
            &mut std::fs::File::open(&path).unwrap(),
            &file_type,
            &fs.parsing_config,
        )
        .unwrap()
        .tag;
        assert_eq!(
            song_tag, read_song_tag,
            "{:?} tag does not match",
            file_type
        );
    }
}

#[test]
fn test_roundtrip_media_file_none_value() {
    let fs = TemporaryFs::new();

    for file_type in SONG_FILE_TYPES {
        let song_tag = SongTag {
            album_artists: vec![],
            track_number: None,
            track_total: None,
            disc_number: None,
            disc_total: None,
            ..Faker.fake()
        };
        let path = fs.create_media_file(
            fs.get_root_path(),
            concat_string!("test.", to_extension(&file_type)),
            song_tag.clone(),
        );
        let read_song_tag = SongInformation::read_from(
            &mut std::fs::File::open(&path).unwrap(),
            &file_type,
            &fs.parsing_config,
        )
        .unwrap()
        .tag;
        assert_eq!(
            song_tag, read_song_tag,
            "{:?} tag does not match",
            file_type
        );
    }
}
