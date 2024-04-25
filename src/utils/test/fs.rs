use std::ffi::OsStr;
use std::fs::*;
use std::io::Write;
use std::path::{Path, PathBuf};

use concat_string::concat_string;
use fake::{Fake, Faker};
use lofty::config::WriteOptions;
use lofty::file::{FileType, TaggedFileExt};
use lofty::tag::{TagExt, TagType};
use nghe_types::constant::SERVER_NAME;
use rand::seq::SliceRandom;
use tempfile::{Builder, TempDir};
use xxhash_rust::xxh3::xxh3_64;

use super::asset::get_media_asset_path;
use crate::config::{ArtConfig, ParsingConfig, TranscodingConfig};
use crate::utils::song::file_type::{to_extension, SONG_FILE_TYPES};
use crate::utils::song::test::SongTag;
use crate::utils::song::{SongInformation, SongLyric};

#[derive(Debug, Clone)]
pub struct SongFsInformation {
    pub tag: SongTag,
    pub lrc: Option<SongLyric>,
    pub music_folder_path: PathBuf,
    pub relative_path: String,
    pub file_hash: u64,
    pub file_size: u32,
}

impl SongFsInformation {
    pub fn absolute_path(&self) -> PathBuf {
        self.music_folder_path.join(&self.relative_path)
    }
}

pub struct TemporaryFs {
    root: TempDir,
    write_option: WriteOptions,

    pub parsing_config: ParsingConfig,
    pub transcoding_config: TranscodingConfig,
    pub art_config: ArtConfig,
}

impl TemporaryFs {
    fn new() -> Self {
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();

        let root = Builder::new().prefix(SERVER_NAME).tempdir().unwrap();
        let write_option = WriteOptions::new().remove_others(true);

        let parsing_config = ParsingConfig::default();
        let transcoding_config = TranscodingConfig {
            cache_path: Some(root.path().canonicalize().unwrap().join("transcoding-cache")),
            ..Default::default()
        };
        let art_config = ArtConfig {
            artist_path: Some(root.path().canonicalize().unwrap().join("art-artist-path")),
            song_path: Some(root.path().canonicalize().unwrap().join("art-song-path")),
        };
        Self { root, write_option, parsing_config, transcoding_config, art_config }
    }

    fn get_absolute_path<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        let root_path = self.root_path();
        let path = path.as_ref();
        if path.is_absolute() {
            if !path.starts_with(root_path) && !path.starts_with(self.canonicalized_root_path()) {
                panic!("path is not a children of root temp directory");
            } else {
                path.into()
            }
        } else {
            root_path.join(path)
        }
    }

    fn create_parent_dir<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        let path = self.get_absolute_path(path);
        self.create_dir(path.parent().unwrap());
        path
    }

    pub fn root_path(&self) -> &Path {
        self.root.path()
    }

    pub fn canonicalized_root_path(&self) -> PathBuf {
        self.root.path().canonicalize().unwrap()
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

    pub fn create_media_file<PM: AsRef<Path>, S: AsRef<str> + ToString>(
        &self,
        music_folder_path: PM,
        relative_path: S,
        song_tag: SongTag,
        generate_lrc: bool,
    ) -> SongFsInformation {
        let tag = song_tag.clone();
        let music_folder_path = self.get_absolute_path(music_folder_path);
        let path = self.create_parent_dir(music_folder_path.join(relative_path.as_ref()));
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
                    .save_to_path(&path, self.write_option)
                    .expect("can not write tag to media file");
            }
            TagType::VorbisComments => {
                song_tag
                    .into_vorbis_comments(&self.parsing_config.vorbis)
                    .save_to_path(&path, self.write_option)
                    .expect("can not write tag to media file");
            }
            _ => unreachable!("media tag type not supported"),
        };

        let file_data = std::fs::read(&path).unwrap();
        let file_hash = xxh3_64(&file_data);
        let file_size = file_data.len() as _;

        let lrc_path = path.with_extension("lrc");
        let lrc = if !generate_lrc {
            if lrc_path.exists() {
                Some(
                    SongLyric::from_str(&std::fs::read_to_string(lrc_path).unwrap(), true).unwrap(),
                )
            } else {
                None
            }
        } else if Faker.fake() {
            let lrc = SongLyric { external: true, ..Faker.fake() };
            std::fs::write(lrc_path, lrc.to_string().as_bytes()).unwrap();
            Some(lrc)
        } else {
            None
        };

        SongFsInformation {
            tag,
            lrc,
            music_folder_path,
            relative_path: relative_path.to_string(),
            file_hash,
            file_size,
        }
    }

    pub fn create_media_files<PM: AsRef<Path>>(
        &self,
        music_folder_path: PM,
        paths: Vec<String>,
        song_tags: Vec<SongTag>,
        generate_lrc: bool,
    ) -> Vec<SongFsInformation> {
        paths
            .into_iter()
            .zip(song_tags)
            .map(|(path, song_tag)| {
                self.create_media_file(&music_folder_path, path, song_tag, generate_lrc)
            })
            .collect()
    }

    pub fn create_random_relative_paths<OS: AsRef<OsStr>>(
        n_path: usize,
        max_depth: usize,
        extensions: &[OS],
    ) -> Vec<String> {
        (0..n_path)
            .map(|_| {
                let ext = extensions.choose(&mut rand::thread_rng()).unwrap();
                Path::new(
                    &fake::vec![String; 1..(max_depth + 1)].join(std::path::MAIN_SEPARATOR_STR),
                )
                .with_extension(ext)
                .to_str()
                .unwrap()
                .to_owned()
            })
            .collect()
    }

    pub fn create_random_paths_media_files<PM: AsRef<Path>, OS: AsRef<OsStr>>(
        &self,
        music_folder_path: PM,
        song_tags: Vec<SongTag>,
        extensions: &[OS],
    ) -> Vec<SongFsInformation> {
        let n_song = song_tags.len();
        self.create_media_files(
            music_folder_path,
            Self::create_random_relative_paths(n_song, 3, extensions),
            song_tags,
            true,
        )
    }

    pub fn join_paths<P: AsRef<Path>>(&self, paths: &[P]) -> Vec<PathBuf> {
        paths.iter().map(|path| self.get_absolute_path(path)).collect()
    }

    pub fn canonicalize_paths<P: AsRef<Path>>(&self, paths: &[P]) -> Vec<PathBuf> {
        paths
            .iter()
            .map(std::fs::canonicalize)
            .collect::<Result<Vec<_>, _>>()
            .expect("can not canonicalize temp path")
    }
}

#[test]
fn test_roundtrip_media_file() {
    let fs = TemporaryFs::default();

    for file_type in SONG_FILE_TYPES {
        let song_tag = Faker.fake::<SongTag>();
        let song_fs_infos = fs.create_media_file(
            fs.root_path(),
            concat_string!("test.", to_extension(&file_type)),
            song_tag.clone(),
            false,
        );
        let read_song_tag = SongInformation::read_from(
            &mut std::fs::File::open(song_fs_infos.absolute_path()).unwrap(),
            file_type,
            None,
            &fs.parsing_config,
        )
        .unwrap()
        .tag;
        assert_eq!(song_tag, read_song_tag, "{:?} tag does not match", file_type);
    }
}

#[test]
fn test_roundtrip_media_file_none_value() {
    let fs = TemporaryFs::default();

    for file_type in SONG_FILE_TYPES {
        let song_tag = SongTag {
            album_artists: vec![],
            track_number: None,
            track_total: None,
            disc_number: None,
            disc_total: None,
            ..Faker.fake()
        };
        let song_fs_infos = fs.create_media_file(
            fs.root_path(),
            concat_string!("test.", to_extension(&file_type)),
            song_tag.clone(),
            false,
        );
        let read_song_tag = SongInformation::read_from(
            &mut std::fs::File::open(song_fs_infos.absolute_path()).unwrap(),
            file_type,
            None,
            &fs.parsing_config,
        )
        .unwrap()
        .tag;
        assert_eq!(song_tag, read_song_tag, "{:?} tag does not match", file_type);
    }
}

impl Default for TemporaryFs {
    fn default() -> Self {
        Self::new()
    }
}
