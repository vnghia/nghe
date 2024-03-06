mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

use super::asset::get_media_asset_path;
use crate::models::*;
use crate::utils::song::file_type::to_extension;
use crate::utils::song::file_type::SONG_FILE_TYPES;
use crate::utils::song::test::song_date_to_string;
use crate::utils::song::test::{id3v2, vorbis_comments, SongTag};
use crate::utils::song::SongInformation;
use crate::{open_subsonic::browsing::refresh_music_folders, DatabasePool};

use concat_string::concat_string;
use fake::{Fake, Faker};
use itertools::Itertools;
use lofty::{
    id3::v2::{Frame, FrameFlags, FrameId, Id3v2Tag, TextInformationFrame},
    ogg::VorbisComments,
    Accessor, FileType, TagExt, TagType, TaggedFileExt,
};
use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::{fs::*, io::Write};
use tempfile::{Builder, TempDir};
use uuid::Uuid;

pub struct TemporaryFs {
    root: TempDir,
}

fn write_id3v2_text_tag(tag: &mut Id3v2Tag, frame_id: FrameId<'static>, value: String) {
    tag.insert(
        Frame::new(
            frame_id,
            TextInformationFrame {
                encoding: lofty::TextEncoding::UTF8,
                value,
            },
            FrameFlags::default(),
        )
        .unwrap(),
    );
}

#[allow(clippy::new_without_default)]
impl TemporaryFs {
    pub const NONE_PATH: Option<&'static PathBuf> = None;

    pub fn new() -> Self {
        Self {
            root: Builder::new()
                .prefix(built_info::PKG_NAME)
                .tempdir()
                .expect("can not create temporary directory"),
        }
    }

    fn get_absolute_path<PR: AsRef<Path>, P: AsRef<Path>>(
        &self,
        root_path: Option<&PR>,
        path: P,
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

    fn create_nested_parent_dir<PR: AsRef<Path>, P: AsRef<Path>>(
        &self,
        root_path: Option<&PR>,
        path: P,
    ) -> PathBuf {
        let path = self.get_absolute_path(root_path, path);
        self.create_nested_dir(root_path, path.parent().unwrap());
        path
    }

    pub fn get_root_path(&self) -> &Path {
        self.root.path()
    }

    pub fn create_nested_dir<PR: AsRef<Path>, P: AsRef<Path>>(
        &self,
        root_path: Option<&PR>,
        path: P,
    ) -> PathBuf {
        let path = self.get_absolute_path(root_path, path);
        create_dir_all(&path).expect("can not create temporary dir");
        path
    }

    pub fn create_dir<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.create_nested_dir(Self::NONE_PATH, path)
    }

    pub fn create_nested_file<PR: AsRef<Path>, P: AsRef<Path>>(
        &self,
        root_path: Option<&PR>,
        path: P,
    ) -> PathBuf {
        let path = self.create_nested_parent_dir(root_path, path);

        File::create(&path)
            .expect("can not open temporary file")
            .write_all(Faker.fake::<String>().as_bytes())
            .expect("can not write to temporary file");
        path
    }

    pub fn create_file<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.create_nested_file(Self::NONE_PATH, path)
    }

    pub fn create_nested_media_file<PR: AsRef<Path>, P: AsRef<Path>>(
        &self,
        root_path: Option<&PR>,
        path: P,
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

        tag.set_title(song_tag.title);
        tag.set_album(song_tag.album);
        if let Some(track_number) = song_tag.track_number {
            tag.set_track(track_number);
        }
        if let Some(track_total) = song_tag.track_total {
            tag.set_track_total(track_total);
        }
        if let Some(disc_number) = song_tag.disc_number {
            tag.set_disk(disc_number);
        }
        if let Some(disc_total) = song_tag.disc_total {
            tag.set_disk_total(disc_total);
        }

        match tag_type {
            TagType::Id3v2 => {
                let multi_value_separator = id3v2::V4_MULTI_VALUE_SEPARATOR.to_string();
                let mut tag = Id3v2Tag::from(tag);
                if !song_tag.artists.is_empty() {
                    tag.set_artist(song_tag.artists.join(&multi_value_separator));
                }
                if !song_tag.album_artists.is_empty() {
                    write_id3v2_text_tag(
                        &mut tag,
                        id3v2::ALBUM_ARTIST_ID,
                        song_tag.album_artists.join(&multi_value_separator),
                    );
                }
                if song_tag.track_number.is_none() && song_tag.track_total.is_some() {
                    write_id3v2_text_tag(
                        &mut tag,
                        id3v2::TRACK_ID,
                        concat_string!("-1/", song_tag.track_total.unwrap().to_string()),
                    );
                }
                if song_tag.disc_number.is_none() && song_tag.disc_total.is_some() {
                    write_id3v2_text_tag(
                        &mut tag,
                        id3v2::DISC_ID,
                        concat_string!("-1/", song_tag.disc_total.unwrap().to_string()),
                    );
                }
                if let Some(date) = song_date_to_string(&song_tag.date) {
                    write_id3v2_text_tag(&mut tag, id3v2::RECORDING_TIME_ID, date);
                }
                if let Some(date) = song_date_to_string(&song_tag.release_date) {
                    write_id3v2_text_tag(&mut tag, id3v2::RELEASE_TIME_ID, date);
                }
                if let Some(date) = song_date_to_string(&song_tag.original_release_date) {
                    write_id3v2_text_tag(&mut tag, id3v2::ORIGINAL_RELEASE_TIME_ID, date);
                }
                if !song_tag.languages.is_empty() {
                    write_id3v2_text_tag(
                        &mut tag,
                        id3v2::LANGUAGE_ID,
                        song_tag
                            .languages
                            .into_iter()
                            .map(|language| language.to_639_3())
                            .join(&multi_value_separator),
                    );
                }
                tag.save_to_path(&path)
                    .expect("can not write tag to media file");
            }
            TagType::VorbisComments => {
                let mut tag = VorbisComments::from(tag);
                song_tag
                    .artists
                    .into_iter()
                    .for_each(|artist| tag.push(vorbis_comments::ARTIST_KEY.to_owned(), artist));
                song_tag.album_artists.into_iter().for_each(|artist| {
                    tag.push(vorbis_comments::ALBUM_ARTIST_KEYS[0].to_owned(), artist)
                });
                if let Some(date) = song_date_to_string(&song_tag.date) {
                    tag.push(vorbis_comments::DATE_KEY.to_owned(), date)
                }
                if let Some(date) = song_date_to_string(&song_tag.original_release_date) {
                    tag.push(vorbis_comments::ORIGINAL_RELEASE_DATE_KEY.to_owned(), date)
                }
                song_tag.languages.into_iter().for_each(|language| {
                    tag.push(
                        vorbis_comments::LANGUAGE.to_owned(),
                        language.to_639_3().to_owned(),
                    )
                });
                tag.save_to_path(&path)
                    .expect("can not write tag to media file");
            }
            _ => unreachable!("media tag type not supported"),
        };

        path
    }

    pub fn create_media_file<P: AsRef<Path>>(&self, path: P, song_tag: SongTag) -> PathBuf {
        self.create_nested_media_file(Self::NONE_PATH, path, song_tag)
    }

    pub fn create_nested_random_paths<PR: AsRef<Path>, OS: AsRef<OsStr>>(
        &self,
        root_path: Option<&PR>,
        n_path: usize,
        max_depth: usize,
        extensions: &[OS],
    ) -> Vec<PathBuf> {
        (0..n_path)
            .map(|_| {
                let ext = extensions.choose(&mut rand::thread_rng()).unwrap();
                self.get_absolute_path(
                    root_path,
                    PathBuf::from(
                        fake::vec![String; 1..(max_depth + 1)].join(std::path::MAIN_SEPARATOR_STR),
                    )
                    .with_extension(ext),
                )
            })
            .collect_vec()
    }

    pub fn create_random_paths<OS: AsRef<OsStr>>(
        &self,
        n_path: usize,
        max_depth: usize,
        extensions: &[OS],
    ) -> Vec<PathBuf> {
        self.create_nested_random_paths(Self::NONE_PATH, n_path, max_depth, extensions)
    }

    pub fn create_nested_media_files<PM: AsRef<Path>, P: AsRef<Path>>(
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
                        self.create_nested_media_file(
                            Some(music_folder_path),
                            path,
                            song_tag.clone(),
                        )
                        .strip_prefix(music_folder_path)
                        .unwrap()
                        .to_path_buf(),
                    ),
                    song_tag,
                )
            })
            .collect::<HashMap<_, _>>()
    }

    pub fn create_nested_random_paths_media_files<PM: AsRef<Path>, OS: AsRef<OsStr>>(
        &self,
        music_folder_id: Uuid,
        music_folder_path: &PM,
        song_tags: Vec<SongTag>,
        extensions: &[OS],
    ) -> HashMap<(Uuid, PathBuf), SongTag> {
        let n_song = song_tags.len();
        self.create_nested_media_files(
            music_folder_id,
            music_folder_path,
            &self.create_nested_random_paths(Some(music_folder_path), n_song, 3, extensions),
            song_tags,
        )
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
            concat_string!("test.", to_extension(&file_type)),
            song_tag.clone(),
        );
        let read_song_tag =
            SongInformation::read_from(&mut std::fs::File::open(&path).unwrap(), &file_type)
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
            concat_string!("test.", to_extension(&file_type)),
            song_tag.clone(),
        );
        let read_song_tag =
            SongInformation::read_from(&mut std::fs::File::open(&path).unwrap(), &file_type)
                .unwrap()
                .tag;
        assert_eq!(
            song_tag, read_song_tag,
            "{:?} tag does not match",
            file_type
        );
    }
}
