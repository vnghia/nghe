#![allow(clippy::struct_field_names)]

use std::borrow::Cow;

use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use fake::{Fake, Faker};
use futures_lite::{StreamExt, stream};
use indexmap::IndexMap;
use itertools::Itertools;
use nghe_api::scan;
use typed_path::{Utf8TypedPath, Utf8TypedPathBuf};
use uuid::Uuid;

use super::Information;
use crate::database::Database;
use crate::file::{self, File, audio, image, lyric};
use crate::filesystem::Trait as _;
use crate::orm::{albums, music_folders, songs};
use crate::scan::scanner;
use crate::test::filesystem::{self, Trait as _};

pub struct Mock<'a> {
    mock: &'a super::Mock,
    music_folder: music_folders::MusicFolder<'static>,
    pub filesystem: IndexMap<Utf8TypedPathBuf, Information<'static, 'static, 'static, 'static>>,
    pub database: IndexMap<Uuid, Information<'static, 'static, 'static, 'static>>,
    pub config: scanner::Config,
}

#[bon::bon]
impl<'a> Mock<'a> {
    pub async fn new(mock: &'a super::Mock, index: usize) -> Self {
        Self {
            mock,
            music_folder: music_folders::table
                .select(music_folders::MusicFolder::as_select())
                .order_by(music_folders::created_at)
                .offset(index.try_into().unwrap())
                .first(&mut mock.get().await)
                .await
                .unwrap(),
            filesystem: IndexMap::new(),
            database: IndexMap::new(),
            config: mock.config.scanner(),
        }
    }

    pub fn database(&self) -> &Database {
        self.mock.database()
    }

    pub fn write_options(&self) -> lofty::config::WriteOptions {
        self.mock.config.lofty_write
    }

    pub fn id(&self) -> Uuid {
        self.music_folder.id
    }

    pub fn path(&self) -> Utf8TypedPath<'_> {
        self.path_str(&self.music_folder.data.path)
    }

    pub fn path_str<'path>(&self, value: &'path (impl AsRef<str> + Sized)) -> Utf8TypedPath<'path> {
        self.to_impl().path().from_str(value)
    }

    pub fn path_string(&self, value: impl Into<String>) -> Utf8TypedPathBuf {
        self.to_impl().path().from_string(value)
    }

    pub fn absolute_path(&self, index: usize) -> Utf8TypedPathBuf {
        self.path().join(self.filesystem.get_index(index).unwrap().0)
    }

    pub fn absolutize(&self, path: impl AsRef<str>) -> Utf8TypedPathBuf {
        let path = self.path_str(&path);
        let music_folder_path = self.path();

        if path.is_absolute() {
            if path.starts_with(music_folder_path) {
                path.to_path_buf()
            } else {
                panic!("Path {path} does not start with music folder path {music_folder_path}")
            }
        } else {
            music_folder_path.join(path)
        }
    }

    pub fn relativize<'b>(&self, path: &'b Utf8TypedPath<'b>) -> Utf8TypedPath<'b> {
        if path.is_absolute() { path.strip_prefix(self.path()).unwrap() } else { *path }
    }

    pub fn to_impl(&self) -> filesystem::Impl<'_> {
        self.mock.to_impl(self.music_folder.data.ty.into())
    }

    #[builder]
    pub async fn add_audio(
        &mut self,
        metadata: Option<audio::Metadata<'static>>,
        song: Option<audio::Song<'static>>,
        album: Option<audio::Album<'static>>,
        artists: Option<audio::Artists<'static>>,
        genres: Option<audio::Genres<'static>>,
        picture: Option<Option<image::Image<'static>>>,
        file_property: Option<file::Property<audio::Format>>,
        external_lyric: Option<Option<lyric::Lyric<'static>>>,
        dir_picture: Option<Option<image::Image<'static>>>,
        relative_path: Option<Cow<'static, str>>,
        song_id: Option<Uuid>,
        #[builder(default = 1)] n_song: usize,
    ) -> &mut Self {
        let builder = Information::builder()
            .maybe_metadata(metadata)
            .maybe_song(song)
            .maybe_album(album)
            .maybe_artists(artists)
            .maybe_genres(genres)
            .maybe_picture(picture)
            .maybe_file_property(file_property)
            .maybe_external_lyric(external_lyric)
            .maybe_dir_picture(dir_picture)
            .maybe_relative_path(relative_path);

        for _ in 0..n_song {
            let information = builder.clone().build();
            let song_id = information.upsert(self, song_id).await;
            self.database.insert(song_id, information);
        }

        self
    }

    #[builder]
    pub async fn add_audio_filesystem(
        &mut self,
        path: Option<impl AsRef<str>>,
        #[builder(default = (0..3).fake::<usize>())] depth: usize,
        #[builder(default = Faker.fake::<audio::Format>())] format: audio::Format,
        metadata: Option<audio::Metadata<'static>>,
        song: Option<audio::Song<'static>>,
        album: Option<audio::Album<'static>>,
        artists: Option<audio::Artists<'static>>,
        genres: Option<audio::Genres<'static>>,
        picture: Option<Option<image::Image<'static>>>,
        external_lyric: Option<Option<lyric::Lyric<'static>>>,
        dir_picture: Option<Option<image::Image<'static>>>,
        #[builder(default = 1)] n_song: usize,
        #[builder(default = true)] scan: bool,
        #[builder(default)] full: scan::start::Full,
        #[builder(default = true)] recompute_dir_picture: bool,
    ) -> &mut Self {
        let builder = Information::builder()
            .maybe_metadata(metadata)
            .maybe_song(song)
            .maybe_album(album)
            .maybe_artists(artists)
            .maybe_genres(genres)
            .maybe_picture(picture)
            .maybe_external_lyric(external_lyric)
            .maybe_dir_picture(dir_picture);

        for _ in 0..n_song {
            let relative_path = if let Some(ref path) = path {
                assert_eq!(n_song, 1, "The same path is supplied for multiple audio");
                self.relativize(&self.path_str(path)).to_path_buf()
            } else {
                self.to_impl().fake_path(depth)
            }
            .with_extension(format.as_ref());

            let information = builder
                .clone()
                .format(format)
                .relative_path(relative_path.to_string().into())
                .build();
            let information = information.dump(self).await;

            self.filesystem.shift_remove(&relative_path);
            self.filesystem.insert(relative_path.clone(), information);
        }

        if scan {
            self.scan(full).run().await.unwrap();
        }

        if recompute_dir_picture {
            let group = self
                .filesystem
                .clone()
                .into_iter()
                .into_group_map_by(|value| value.0.parent().unwrap().to_path_buf());
            for (parent, files) in group {
                let dir_picture = image::Image::scan_filesystem(
                    &self.to_impl(),
                    &self.config.cover_art,
                    self.path().join(parent).to_path(),
                )
                .await;
                for (path, information) in files {
                    let information =
                        Information { dir_picture: dir_picture.clone(), ..information };
                    self.filesystem.insert(path, information);
                }
            }
        }

        self
    }

    pub async fn add_audio_artist(
        &mut self,
        songs: impl IntoIterator<Item = audio::Artist<'static>>,
        albums: impl IntoIterator<Item = audio::Artist<'static>>,
        compilation: bool,
        n_song: usize,
    ) {
        self.add_audio()
            .artists(audio::Artists {
                song: songs.into_iter().collect(),
                album: albums.into_iter().collect(),
                compilation,
            })
            .n_song(n_song)
            .call()
            .await;
    }

    #[builder]
    pub async fn remove_audio_filesystem(
        &mut self,
        path: Option<impl AsRef<str>>,
        #[builder(default = 0)] index: usize,
        #[builder(default = true)] scan: bool,
        #[builder(default)] full: scan::start::Full,
    ) -> &mut Self {
        if let Some(path) = path {
            let absolute_path = self.absolutize(path);
            let absolute_path = absolute_path.to_path();
            let relative_path = self.relativize(&absolute_path).to_path_buf();
            self.to_impl().delete(absolute_path).await;
            self.filesystem.shift_remove(&relative_path);
        } else if let Some((relative_path, _)) = self.filesystem.shift_remove_index(index) {
            self.to_impl().delete(self.absolutize(relative_path).to_path()).await;
        }

        if scan {
            self.scan(full).run().await.unwrap();
        }

        self
    }

    pub async fn file(&self, path: Utf8TypedPath<'_>, format: audio::Format) -> audio::File {
        let path = self.absolutize(path).with_extension(format.as_ref());
        File::new(format, self.to_impl().read(path.to_path()).await.unwrap())
            .unwrap()
            .audio(self.config.lofty)
            .unwrap()
    }

    pub fn scan(&self, full: scan::start::Full) -> scanner::Scanner<'_, '_, '_> {
        scanner::Scanner::new_orm(
            self.mock.database(),
            self.mock.filesystem(),
            self.config.clone(),
            self.mock.informant.clone(),
            music_folders::MusicFolder {
                id: self.music_folder.id,
                data: music_folders::Data {
                    path: self.music_folder.data.path.as_str().into(),
                    ty: self.music_folder.data.ty,
                },
            },
            full,
        )
        .unwrap()
    }

    async fn optional_song_id_filesystem(&self, index: usize) -> Option<Uuid> {
        let path = self.filesystem.get_index(index).unwrap().0.as_str();
        albums::table
            .inner_join(songs::table)
            .filter(albums::music_folder_id.eq(self.music_folder.id))
            .filter(songs::relative_path.eq(path))
            .select(songs::id)
            .get_result(&mut self.mock.get().await)
            .await
            .optional()
            .unwrap()
    }

    pub fn song_id(&self, index: usize) -> Uuid {
        *self.database.get_index(index).unwrap().0
    }

    pub async fn song_id_filesystem(&self, index: usize) -> Uuid {
        self.optional_song_id_filesystem(index).await.unwrap()
    }

    pub async fn query_filesystem(
        &self,
    ) -> IndexMap<Utf8TypedPathBuf, Information<'static, 'static, 'static, 'static>> {
        let song_ids: Vec<_> = stream::iter(0..self.filesystem.len())
            .then(async |index| self.optional_song_id_filesystem(index).await)
            .filter_map(std::convert::identity)
            .collect()
            .await;
        stream::iter(song_ids)
            .then(async |id| {
                let mock = Information::query(self.mock, id).await;
                (self.path_str(&mock.relative_path).to_path_buf(), mock)
            })
            .collect()
            .await
    }
}

mod duration {
    use audio::duration::Trait;

    use super::*;

    impl Trait for IndexMap<Uuid, Information<'static, 'static, 'static, 'static>> {
        fn duration(&self) -> audio::Duration {
            self.values().map(|information| information.information.property.duration).sum()
        }
    }
}
