use std::io::{Cursor, Write};

use concat_string::concat_string;
use fake::{Fake, Faker};
use futures::{stream, StreamExt};
use lofty::config::WriteOptions;
use lofty::tag::{TagExt, TagType};
use nghe_types::constant::SERVER_NAME;
use rand::prelude::SliceRandom;
use tempfile::{Builder, TempDir};
use xxhash_rust::xxh3::xxh3_64;

use super::asset::get_media_asset_path;
use crate::config::{ArtConfig, ParsingConfig, TranscodingConfig};
use crate::utils::path::{LocalPath, PathTest, PathTrait};
use crate::utils::song::file_type::{to_extension, SONG_FILE_TYPES};
use crate::utils::song::test::SongTag;
use crate::utils::song::{SongInformation, SongLyric};

#[derive(Debug, Clone)]
pub struct SongFsInformation {
    pub tag: SongTag,
    pub music_folder_path: String,
    pub relative_path: String,
    pub lrc: Option<SongLyric>,
    pub file_hash: u64,
    pub file_size: u32,
}

impl SongFsInformation {
    pub fn path(&self, root: &TemporaryFsRoot) -> Box<impl PathTrait + PathTest + ToString> {
        Box::new(LocalPath::new(root, Some(&self.music_folder_path)).join(&self.relative_path))
    }
}

pub struct TemporaryFsRoot {
    pub local: TempDir,
}

pub struct TemporaryFs {
    pub root: TemporaryFsRoot,

    pub write_option: WriteOptions,
    pub parsing_config: ParsingConfig,
    pub transcoding_config: TranscodingConfig,
    pub art_config: ArtConfig,
}

impl TemporaryFs {
    fn new() -> Self {
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();

        let root = TemporaryFsRoot {
            local: Builder::new().prefix(&concat_string!(SERVER_NAME, "-")).tempdir().unwrap(),
        };

        let write_option = WriteOptions::new().remove_others(true);
        let parsing_config = ParsingConfig::default();
        let transcoding_config = TranscodingConfig {
            cache_path: Some(root.local.path().canonicalize().unwrap().join("transcoding-cache")),
            ..Default::default()
        };
        let art_config = ArtConfig {
            artist_dir: Some(root.local.path().canonicalize().unwrap().join("art-artist-path")),
            song_dir: Some(root.local.path().canonicalize().unwrap().join("art-song-path")),
        };
        Self { root, write_option, parsing_config, transcoding_config, art_config }
    }

    fn to_path<P: PathTest>(&self, rel_path: &str) -> P {
        P::new(&self.root, None).join(rel_path)
    }

    pub async fn mkdir<P: PathTest + ToString>(&self, rel_path: &str) -> P {
        let path: P = self.to_path(rel_path);
        path.mkdir().await;
        path
    }

    pub async fn mkfile<P: PathTest>(&self, rel_path: &str) -> P {
        let content: String = Faker.fake();
        let path: P = self.to_path(rel_path);
        path.write(content).await;
        path
    }

    pub async fn mksong<P: PathTrait + PathTest>(
        &self,
        music_folder_path: &str,
        rel_path: &str,
        tag: SongTag,
        mklrc: bool,
    ) -> SongFsInformation {
        let path = P::new(&self.root, Some(music_folder_path)).join(rel_path);
        let file_type = path.file_type();
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
        path.write(&file_data).await;
        let file_hash = xxh3_64(&file_data);
        let file_size = file_data.len() as _;

        let lrc = if !mklrc {
            path.read_lrc().await.map(|s| SongLyric::from_str(&s, true).unwrap()).ok()
        } else if Faker.fake() {
            let lrc = SongLyric { external: true, ..Faker.fake() };
            path.lrc().write(lrc.to_string()).await;
            Some(lrc)
        } else {
            None
        };

        SongFsInformation {
            tag,
            lrc,
            music_folder_path: music_folder_path.into(),
            relative_path: rel_path.into(),
            file_hash,
            file_size,
        }
    }

    pub async fn mksongs<P: PathTrait + PathTest, S: AsRef<str>>(
        &self,
        music_folder_path: &str,
        paths: &[S],
        tags: Vec<SongTag>,
        mklrc: bool,
    ) -> Vec<SongFsInformation> {
        stream::iter(paths)
            .zip(stream::iter(tags))
            .then(move |(path, tag)| async move {
                self.mksong::<P>(music_folder_path, path.as_ref(), tag, mklrc).await
            })
            .collect()
            .await
    }

    pub fn mkrelpaths<P: PathTrait, S: AsRef<str>>(
        &self,
        n_path: usize,
        max_depth: usize,
        exts: &[S],
    ) -> Vec<String> {
        (0..n_path)
            .map(|_| {
                let ext = exts.choose(&mut rand::thread_rng()).unwrap();
                concat_string!(
                    fake::vec![String; 1..(max_depth + 1)].join(P::PATH_SEPARATOR),
                    ".",
                    ext
                )
            })
            .collect()
    }

    pub async fn mkpathssongs<P: PathTrait + PathTest, S: AsRef<str>>(
        &self,
        music_folder_path: &str,
        song_tags: Vec<SongTag>,
        exts: &[S],
    ) -> Vec<SongFsInformation> {
        self.mksongs::<P, _>(
            music_folder_path,
            &self.mkrelpaths::<P, S>(song_tags.len(), 3, exts),
            song_tags,
            true,
        )
        .await
    }
}

impl Default for TemporaryFs {
    fn default() -> Self {
        Self::new()
    }
}

#[tokio::test]
async fn test_roundtrip_media_file() {
    let fs = TemporaryFs::default();

    for file_type in SONG_FILE_TYPES {
        let song_tag = Faker.fake::<SongTag>();
        let song_fs_infos = fs
            .mksong::<LocalPath>(
                &fs.mkdir::<LocalPath>(&Faker.fake::<String>()).await.to_string(),
                &concat_string!("test.", to_extension(&file_type)),
                song_tag.clone(),
                false,
            )
            .await;
        let read_song_tag = SongInformation::read_from(
            &mut Cursor::new(song_fs_infos.path(&fs.root).read().await.unwrap()),
            file_type,
            None,
            &fs.parsing_config,
        )
        .unwrap()
        .tag;
        assert_eq!(song_tag, read_song_tag, "{:?} tag does not match", file_type);
    }
}

#[tokio::test]
async fn test_roundtrip_media_file_none_value() {
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
        let song_fs_infos = fs
            .mksong::<LocalPath>(
                &fs.mkdir::<LocalPath>(&Faker.fake::<String>()).await.to_string(),
                &concat_string!("test.", to_extension(&file_type)),
                song_tag.clone(),
                false,
            )
            .await;
        let read_song_tag = SongInformation::read_from(
            &mut Cursor::new(song_fs_infos.path(&fs.root).read().await.unwrap()),
            file_type,
            None,
            &fs.parsing_config,
        )
        .unwrap()
        .tag;
        assert_eq!(song_tag, read_song_tag, "{:?} tag does not match", file_type);
    }
}
