use std::any::Any;
use std::io::{Cursor, Write};

use anyhow::Result;
use concat_string::concat_string;
use fake::{Fake, Faker};
use futures::{stream, StreamExt};
use lofty::config::WriteOptions;
use lofty::tag::{TagExt, TagType};
use rand::distributions::{Alphanumeric, DistString};
use rand::prelude::SliceRandom;
use typed_path::Utf8Path;
use xxhash_rust::xxh3::xxh3_64;

use super::{TemporaryLocalFs, TemporaryS3Fs};
use crate::config::{ArtConfig, ParsingConfig, TranscodingConfig};
use crate::models::*;
use crate::utils::fs::{FsTrait, LocalFs, S3Fs};
use crate::utils::song::file_type::{to_extension, SUPPORTED_EXTENSIONS};
use crate::utils::song::test::SongTag;
use crate::utils::song::{SongInformation, SongLyric};
use crate::utils::test::asset::get_media_asset_path;

#[derive(Debug, Clone)]
pub struct SongFsInformation {
    pub tag: SongTag,
    pub music_folder_path: String,
    pub relative_path: String,
    pub lrc: Option<SongLyric>,
    pub file_hash: u64,
    pub file_size: u32,
    pub fs: music_folders::FsType,
}

pub fn strip_prefix<'a, Fs: FsTrait>(path: &'a str, base: &str) -> &'a str
where
    Fs::E: 'a,
{
    Utf8Path::<Fs::E>::new(path).strip_prefix(base).unwrap().as_str()
}

pub fn extension<'a, Fs: FsTrait>(path: &'a str) -> &'a str
where
    Fs::E: 'a,
{
    Utf8Path::<Fs::E>::new(path).extension().unwrap()
}

pub fn with_extension<Fs: FsTrait>(path: &str, extension: &str) -> String {
    Utf8Path::<Fs::E>::new(path).with_extension(extension).into_string()
}

#[async_trait::async_trait]
pub trait TemporaryFsTrait {
    fn prefix(&self) -> &str;
    fn fs(&self) -> &dyn Any;

    fn join(&self, base: &str, path: &str) -> String;
    fn strip_prefix<'a>(&self, path: &'a str, base: &str) -> &'a str;
    fn extension<'a>(&self, path: &'a str) -> &'a str;
    fn with_extension(&self, path: &str, ext: &str) -> String;

    async fn read(&self, path: &str) -> Result<Vec<u8>>;
    async fn read_to_string(&self, path: &str) -> Result<String>;
    async fn read_to_transcoding_input(&self, path: String) -> String;

    async fn mkdir(&self, path: &str);
    async fn write(&self, path: &str, data: &[u8]);
    async fn remove(&self, path: &str);
}

pub struct TemporaryFs {
    fs: [Box<dyn TemporaryFsTrait>; 2],

    pub write_option: WriteOptions,
    pub parsing_config: ParsingConfig,
    pub transcoding_config: TranscodingConfig,
    pub art_config: ArtConfig,
}

impl TemporaryFs {
    pub async fn new() -> Self {
        let local = TemporaryLocalFs::default();

        let write_option = WriteOptions::new().remove_others(true);
        let parsing_config = ParsingConfig::default();
        let transcoding_config = TranscodingConfig {
            cache_dir: Some(
                local
                    .root
                    .path()
                    .canonicalize()
                    .unwrap()
                    .join("transcoding-cache")
                    .to_str()
                    .unwrap()
                    .into(),
            ),
            ..Default::default()
        };
        let art_config = ArtConfig {
            artist_dir: Some(
                local
                    .root
                    .path()
                    .canonicalize()
                    .unwrap()
                    .join("art-artist-path")
                    .to_str()
                    .unwrap()
                    .into(),
            ),
            song_dir: Some(
                local
                    .root
                    .path()
                    .canonicalize()
                    .unwrap()
                    .join("art-song-path")
                    .to_str()
                    .unwrap()
                    .into(),
            ),
        };

        Self {
            fs: [Box::new(local), Box::new(TemporaryS3Fs::new().await)],
            write_option,
            parsing_config,
            transcoding_config,
            art_config,
        }
    }

    fn absolute_path(&self, fs: &dyn TemporaryFsTrait, path: &str) -> String {
        if path.starts_with(fs.prefix()) { path.to_string() } else { fs.join(fs.prefix(), path) }
    }

    pub fn fs(&self, fs_type: music_folders::FsType) -> &dyn TemporaryFsTrait {
        self.fs[(fs_type as i16 - 1) as usize].as_ref()
    }

    pub fn local(&self) -> &LocalFs {
        self.fs[0].fs().downcast_ref::<LocalFs>().unwrap()
    }

    pub fn s3(&self) -> &S3Fs {
        self.s3_option().unwrap()
    }

    pub fn s3_option(&self) -> Option<&S3Fs> {
        self.fs[1].fs().downcast_ref::<S3Fs>()
    }

    pub fn fake_fs_name() -> String {
        Alphanumeric.sample_string(&mut rand::thread_rng(), (5..10).fake()).to_ascii_lowercase()
    }

    pub fn prefix(&self, fs: music_folders::FsType) -> &str {
        self.fs(fs).prefix()
    }

    pub fn strip_prefix(&self, fs: music_folders::FsType, path: &str, base: &str) -> String {
        let fs = self.fs(fs);
        fs.strip_prefix(&self.absolute_path(fs, path), &self.absolute_path(fs, base)).into()
    }

    pub fn extension<'a>(&self, fs: music_folders::FsType, path: &'a str) -> &'a str {
        let fs = self.fs(fs);
        fs.extension(path)
    }

    pub fn with_extension(&self, fs: music_folders::FsType, path: &str, ext: &str) -> String {
        let fs = self.fs(fs);
        fs.with_extension(&self.absolute_path(fs, path), ext)
    }

    pub fn join(&self, fs: music_folders::FsType, base: &str, path: &str) -> String {
        let fs = self.fs(fs);
        fs.join(&self.absolute_path(fs, base), path)
    }

    pub fn song_absolute_path(&self, info: &SongFsInformation) -> String {
        self.join(info.fs, &info.music_folder_path, &info.relative_path)
    }

    pub async fn read(&self, fs: music_folders::FsType, path: &str) -> Vec<u8> {
        let fs = self.fs(fs);
        let path = self.absolute_path(fs, path);
        fs.read(&path).await.unwrap()
    }

    pub async fn read_to_string(&self, fs: music_folders::FsType, path: &str) -> String {
        let fs = self.fs(fs);
        let path = self.absolute_path(fs, path);
        fs.read_to_string(&path).await.unwrap()
    }

    pub async fn read_to_transcoding_input(&self, info: &SongFsInformation) -> String {
        let path = self.song_absolute_path(info);
        let fs = self.fs(info.fs);
        fs.read_to_transcoding_input(path).await
    }

    pub async fn write(&self, fs: music_folders::FsType, path: &str, data: impl AsRef<[u8]>) {
        let fs = self.fs(fs);
        let path = self.absolute_path(fs, path);
        fs.write(&path, data.as_ref()).await;
    }

    pub async fn read_song(&self, info: &SongFsInformation) -> Vec<u8> {
        self.read(info.fs, &self.song_absolute_path(info)).await
    }

    pub async fn mkdir(&self, fs: music_folders::FsType, path: &str) -> String {
        let fs = self.fs(fs);
        let path = self.absolute_path(fs, path);
        fs.mkdir(&path).await;
        path
    }

    pub async fn mkfile(&self, fs: music_folders::FsType, path: &str) -> String {
        let fs = self.fs(fs);
        let path = self.absolute_path(fs, path);
        fs.write(&path, Faker.fake::<String>().as_bytes()).await;
        path
    }

    pub async fn remove(&self, fs: music_folders::FsType, path: &str) {
        let fs = self.fs(fs);
        fs.remove(&self.absolute_path(fs, path)).await;
    }

    pub async fn mksong(
        &self,
        fs: music_folders::FsType,
        music_folder_path: &str,
        relative_path: &str,
        tag: SongTag,
        mklrc: bool,
    ) -> SongFsInformation {
        let fs_idx = fs;
        let fs = self.fs(fs);

        let path = fs.join(&self.absolute_path(fs, music_folder_path), relative_path);
        let file_type = *SUPPORTED_EXTENSIONS.get(fs.extension(&path)).unwrap();
        let mut tag_file =
            Cursor::new(tokio::fs::read(get_media_asset_path(&file_type)).await.unwrap());
        let tag_type = file_type.primary_tag_type();

        let song_tag = tag.clone();
        match tag_type {
            TagType::Id3v2 => {
                song_tag
                    .into_id3v2(&self.parsing_config.id3v2)
                    .save_to(&mut tag_file, self.write_option)
                    .unwrap();
            }
            TagType::VorbisComments => {
                song_tag
                    .into_vorbis_comments(&self.parsing_config.vorbis)
                    .save_to(&mut tag_file, self.write_option)
                    .unwrap();
            }
            _ => unreachable!(),
        };
        tag_file.flush().unwrap();
        tag_file.set_position(0);

        let file_data = tag_file.into_inner();
        fs.write(&path, &file_data).await;
        let file_hash = xxh3_64(&file_data);
        let file_size = file_data.len() as _;

        let lrc = if !mklrc {
            fs.read_to_string(&fs.with_extension(&path, "lrc"))
                .await
                .map(|s| SongLyric::from_str(&s, true).unwrap())
                .ok()
        } else if Faker.fake() {
            let lrc = SongLyric { external: true, ..Faker.fake() };
            fs.write(&fs.with_extension(&path, "lrc"), lrc.to_string().as_bytes()).await;
            Some(lrc)
        } else {
            None
        };

        SongFsInformation {
            tag,
            lrc,
            music_folder_path: music_folder_path.to_string(),
            relative_path: relative_path.to_string(),
            file_hash,
            file_size,
            fs: fs_idx,
        }
    }

    pub async fn mksongs(
        &self,
        fs: music_folders::FsType,
        music_folder_path: &str,
        relative_paths: &[impl AsRef<str>],
        tags: Vec<SongTag>,
        mklrc: bool,
    ) -> Vec<SongFsInformation> {
        stream::iter(relative_paths)
            .zip(stream::iter(tags))
            .then(move |(relative_path, tag)| async move {
                self.mksong(fs, music_folder_path, relative_path.as_ref(), tag, mklrc).await
            })
            .collect()
            .await
    }

    pub fn mkrelpaths(
        &self,
        fs: music_folders::FsType,
        n_path: usize,
        max_depth: usize,
        exts: &[impl AsRef<str>],
    ) -> Vec<String> {
        let fs = self.fs(fs);

        (0..n_path)
            .map(|_| {
                let ext = exts.choose(&mut rand::thread_rng()).unwrap();
                fs.with_extension(
                    &fake::vec![String; 1..(max_depth + 1)]
                        .into_iter()
                        .reduce(|base, path| fs.join(&base, &path))
                        .unwrap(),
                    ext.as_ref(),
                )
            })
            .collect()
    }

    pub async fn mkpathssongs(
        &self,
        fs: music_folders::FsType,
        music_folder_path: &str,
        song_tags: Vec<SongTag>,
        exts: &[impl AsRef<str>],
    ) -> Vec<SongFsInformation> {
        self.mksongs(
            fs,
            music_folder_path,
            &self.mkrelpaths(fs, song_tags.len(), 3, exts),
            song_tags,
            true,
        )
        .await
    }
}

#[tokio::test]
async fn test_roundtrip_media_file() {
    let fs = TemporaryFs::new().await;

    for (idx, fs_impl) in fs.fs.iter().enumerate() {
        let fs_type = music_folders::FsType::from_repr((idx + 1) as _).unwrap();
        for file_type in SUPPORTED_EXTENSIONS.values().copied() {
            let song_tag = Faker.fake::<SongTag>();
            let song_fs_info = fs
                .mksong(
                    fs_type,
                    &fs.mkdir(fs_type, &TemporaryFs::fake_fs_name()).await,
                    &concat_string!("test.", to_extension(&file_type)),
                    song_tag.clone(),
                    false,
                )
                .await;
            let read_song_tag = SongInformation::read_from(
                &mut Cursor::new(
                    fs_impl.read(&fs.song_absolute_path(&song_fs_info)).await.unwrap(),
                ),
                file_type,
                None,
                &fs.parsing_config,
            )
            .unwrap()
            .tag;
            assert_eq!(
                song_tag, read_song_tag,
                "{:?} tag for fs {:?} does not match",
                file_type, fs_type
            );
        }
    }
}

#[tokio::test]
async fn test_roundtrip_media_file_none_value() {
    let fs = TemporaryFs::new().await;

    for (idx, fs_impl) in fs.fs.iter().enumerate() {
        let fs_type = music_folders::FsType::from_repr((idx + 1) as _).unwrap();
        for file_type in SUPPORTED_EXTENSIONS.values().copied() {
            let song_tag = SongTag {
                album_artists: vec![],
                track_number: None,
                track_total: None,
                disc_number: None,
                disc_total: None,
                ..Faker.fake()
            };
            let song_fs_info = fs
                .mksong(
                    fs_type,
                    &fs.mkdir(fs_type, &TemporaryFs::fake_fs_name()).await,
                    &concat_string!("test.", to_extension(&file_type)),
                    song_tag.clone(),
                    false,
                )
                .await;
            let read_song_tag = SongInformation::read_from(
                &mut Cursor::new(
                    fs_impl.read(&fs.song_absolute_path(&song_fs_info)).await.unwrap(),
                ),
                file_type,
                None,
                &fs.parsing_config,
            )
            .unwrap()
            .tag;
            assert_eq!(
                song_tag, read_song_tag,
                "{:?} tag for fs {:?} does not match",
                file_type, fs_type
            );
        }
    }
}
