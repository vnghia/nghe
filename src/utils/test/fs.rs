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
    id3::v2::Id3v2Tag, ogg::VorbisComments, Accessor, FileType, TagExt, TagType, TaggedFileExt,
};
use std::path::{Path, PathBuf};
use std::{fs::*, io::Write};
use tempdir::TempDir;

pub struct TemporaryFs {
    root: TempDir,
}

#[allow(clippy::new_without_default)]
impl TemporaryFs {
    pub fn new() -> Self {
        Self {
            root: TempDir::new(built_info::PKG_NAME).expect("can not create temporary directory"),
        }
    }

    fn get_absolute_path<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        if path.as_ref().is_absolute() {
            if !path.as_ref().starts_with(self.get_root_path()) {
                panic!("path is not a children of root temp directory");
            } else {
                path.as_ref().into()
            }
        } else {
            self.get_root_path().join(path)
        }
    }

    pub fn create_dir<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        let path = self.get_absolute_path(path);
        create_dir_all(&path).expect("can not create temporary dir");
        path
    }

    pub fn create_file<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        let path = self.get_absolute_path(path);
        self.create_dir(path.parent().unwrap());

        File::create(&path)
            .expect("can not open temporary file")
            .write_all(Faker.fake::<String>().as_bytes())
            .expect("can not write to temporary file");
        path
    }

    pub fn create_media_file<P: AsRef<Path>>(
        &self,
        path: P,
        file_type: &FileType,
        song_tag: SongTag,
    ) -> PathBuf {
        let path = self.get_absolute_path(path);
        self.create_dir(path.parent().unwrap());
        std::fs::copy(get_media_asset_path(file_type), &path)
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
                tag.save_to_path(&path)
                    .expect("can not write tag to media file");
            }
            _ => unreachable!("media tag type not supported"),
        };

        path
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

    pub fn get_root_path(&self) -> &Path {
        self.root.path()
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
            &file_type,
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
