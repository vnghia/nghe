mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

use super::asset::get_media_asset_path;
use crate::models::*;
use crate::utils::song::file_type::to_extension;
use crate::utils::song::file_type::SONG_FILE_TYPES;
use crate::utils::song::tag::SongTag;
use crate::{open_subsonic::browsing::refresh_music_folders, DatabasePool};

use concat_string::concat_string;
use fake::{Fake, Faker};
use itertools::Itertools;
use lofty::{
    id3::v2::{Frame, FrameFlags, Id3v2Tag, TextInformationFrame},
    ogg::VorbisComments,
    Accessor, FileType, TagExt, TagType, TaggedFileExt,
};
use rand::seq::SliceRandom;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::{fs::*, io::Write};
use tempdir::TempDir;

pub struct TemporaryFs {
    root: TempDir,
}

#[allow(clippy::new_without_default)]
impl TemporaryFs {
    pub const NONE_PATH: Option<&'static PathBuf> = None;

    pub fn new() -> Self {
        Self {
            root: TempDir::new(built_info::PKG_NAME).expect("can not create temporary directory"),
        }
    }

    fn get_absolute_path<TR: AsRef<Path>, TP: AsRef<Path>>(
        &self,
        root_path: Option<&TR>,
        path: TP,
    ) -> PathBuf {
        let root_path = match root_path {
            Some(root_path) => root_path.as_ref(),
            None => self.get_root_path(),
        };

        if path.as_ref().is_absolute() {
            if !path.as_ref().starts_with(root_path) {
                panic!("path is not a children of root temp directory");
            } else {
                path.as_ref().into()
            }
        } else {
            root_path.join(path)
        }
    }

    fn create_nested_parent_dir<TR: AsRef<Path>, TP: AsRef<Path>>(
        &self,
        root_path: Option<&TR>,
        path: TP,
    ) -> PathBuf {
        let path = self.get_absolute_path(root_path, path);
        self.create_nested_dir(root_path, path.parent().unwrap());
        path
    }

    pub fn get_root_path(&self) -> &Path {
        self.root.path()
    }

    pub fn create_nested_dir<TR: AsRef<Path>, TP: AsRef<Path>>(
        &self,
        root_path: Option<&TR>,
        path: TP,
    ) -> PathBuf {
        let path = self.get_absolute_path(root_path, path);
        create_dir_all(&path).expect("can not create temporary dir");
        path
    }

    pub fn create_dir<TP: AsRef<Path>>(&self, path: TP) -> PathBuf {
        self.create_nested_dir(Self::NONE_PATH, path)
    }

    pub fn create_nested_file<TR: AsRef<Path>, TP: AsRef<Path>>(
        &self,
        root_path: Option<&TR>,
        path: TP,
    ) -> PathBuf {
        let path = self.create_nested_parent_dir(root_path, path);

        File::create(&path)
            .expect("can not open temporary file")
            .write_all(Faker.fake::<String>().as_bytes())
            .expect("can not write to temporary file");
        path
    }

    pub fn create_file<TP: AsRef<Path>>(&self, path: TP) -> PathBuf {
        self.create_nested_file(Self::NONE_PATH, path)
    }

    pub fn create_nested_media_file<TR: AsRef<Path>, TP: AsRef<Path>>(
        &self,
        root_path: Option<&TR>,
        path: TP,
        song_tag: SongTag,
    ) -> PathBuf {
        let path = self.create_nested_parent_dir(root_path, path);
        let file_type = FileType::from_path(&path).unwrap();

        std::fs::copy(get_media_asset_path(&file_type), &path)
            .expect("can not copy original media file to temp directory");

        let tag_type = lofty::read_from_path(&path)
            .expect("can not read original media file")
            .primary_tag_type();
        let mut tag = lofty::Tag::new(tag_type);
        tag.set_title(song_tag.title.clone());
        tag.set_album(song_tag.album.clone());

        match tag_type {
            TagType::Id3v2 => {
                let mut tag = Id3v2Tag::from(tag);
                tag.set_artist(song_tag.artists.join("\0"));
                if !song_tag.album_artists.is_empty() {
                    tag.insert(
                        Frame::new(
                            "TPE2",
                            TextInformationFrame {
                                encoding: lofty::TextEncoding::UTF8,
                                value: song_tag.album_artists.join("\0"),
                            },
                            FrameFlags::default(),
                        )
                        .unwrap(),
                    );
                }
                tag.save_to_path(&path)
                    .expect("can not write tag to media file");
            }
            TagType::VorbisComments => {
                let mut tag = VorbisComments::from(tag);
                song_tag
                    .artists
                    .iter()
                    .cloned()
                    .for_each(|artist| tag.push("ARTIST".to_owned(), artist));
                song_tag
                    .album_artists
                    .iter()
                    .cloned()
                    .for_each(|artist| tag.push("ALBUMARTIST".to_owned(), artist));
                tag.save_to_path(&path)
                    .expect("can not write tag to media file");
            }
            _ => unreachable!("media tag type not supported"),
        };

        path
    }

    pub fn create_media_file<TP: AsRef<Path>>(&self, path: TP, song_tag: SongTag) -> PathBuf {
        self.create_nested_media_file(Self::NONE_PATH, path, song_tag)
    }

    pub fn create_nested_random_paths<TP: AsRef<Path>, TE: AsRef<OsStr>>(
        &self,
        root_path: Option<&TP>,
        n_path: u8,
        max_depth: u8,
        extensions: &[TE],
    ) -> Vec<(PathBuf, Option<FileType>)> {
        (0..n_path)
            .into_iter()
            .map(|_| {
                let ext = extensions.choose(&mut rand::thread_rng()).unwrap();
                (
                    self.get_absolute_path(
                        root_path,
                        PathBuf::from(
                            fake::vec![String; 1..(max_depth as usize + 1)]
                                .join(std::path::MAIN_SEPARATOR_STR),
                        )
                        .with_extension(ext),
                    ),
                    FileType::from_ext(ext),
                )
            })
            .collect_vec()
    }

    pub fn create_random_paths<TE: AsRef<OsStr>>(
        &self,
        n_path: u8,
        max_depth: u8,
        extensions: &[TE],
    ) -> Vec<(PathBuf, Option<FileType>)> {
        self.create_nested_random_paths(Self::NONE_PATH, n_path, max_depth, extensions)
    }

    pub fn join_paths<P: AsRef<Path>>(&self, paths: &[P]) -> Vec<PathBuf> {
        paths
            .iter()
            .map(|path| self.get_absolute_path(Self::NONE_PATH, path))
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
        n_folder: u8,
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
            concat_string!("test.", to_extension(&file_type)),
            song_tag.clone(),
        );
        let read_song_tag = SongTag::parse(std::fs::read(&path).unwrap(), file_type).unwrap();
        assert_eq!(
            song_tag, read_song_tag,
            "{:?} tag does not match",
            file_type
        );
    }
}
