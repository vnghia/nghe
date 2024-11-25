#![allow(clippy::struct_field_names)]

use std::borrow::Cow;
use std::io::{Cursor, Write};

use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use fake::{Fake, Faker};
use futures_lite::{stream, StreamExt};
use indexmap::IndexMap;
use typed_path::{Utf8TypedPath, Utf8TypedPathBuf};
use uuid::Uuid;

use super::Information;
use crate::database::Database;
use crate::file::{self, audio, picture, File};
use crate::filesystem::Trait as _;
use crate::orm::{albums, music_folders, songs};
use crate::scan::scanner;
use crate::test::assets;
use crate::test::file::audio::dump::Metadata as _;
use crate::test::filesystem::{self, Trait as _};

pub struct Mock<'a> {
    mock: &'a super::Mock,
    music_folder: music_folders::MusicFolder<'static>,
    pub filesystem: IndexMap<Utf8TypedPathBuf, Information<'static, 'static>>,
    pub database: IndexMap<Uuid, Information<'static, 'static>>,
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
        picture: Option<Option<picture::Picture<'static, 'static>>>,
        #[builder(default = 1)] n_song: usize,
    ) -> &mut Self {
        for _ in 0..n_song {
            let relative_path = Faker.fake::<String>();
            let information = super::Mock::information()
                .maybe_metadata(metadata.clone())
                .maybe_song(song.clone())
                .maybe_album(album.clone())
                .maybe_artists(artists.clone())
                .maybe_genres(genres.clone())
                .maybe_picture(picture.clone())
                .call();
            let song_id = information
                .upsert(self.mock.database(), &self.config, self.id().into(), &relative_path, None)
                .await
                .unwrap();
            self.database
                .insert(song_id, Information { information, relative_path: relative_path.into() });
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
        picture: Option<Option<picture::Picture<'static, 'static>>>,
        #[builder(default = 1)] n_song: usize,
        #[builder(default = true)] scan: bool,
    ) -> &mut Self {
        for _ in 0..n_song {
            let path = if let Some(ref path) = path {
                assert_eq!(n_song, 1, "The same path is supplied for multiple audio");
                self.absolutize(path)
            } else {
                self.absolutize(self.to_impl().fake_path(depth))
            }
            .with_extension(format.as_ref());

            let data = tokio::fs::read(assets::path(format).as_str()).await.unwrap();
            let mut asset = Cursor::new(data.clone());
            let mut file = File::new(format, data).unwrap().audio(self.config.lofty).unwrap();
            asset.set_position(0);

            let metadata = super::Mock::information()
                .maybe_metadata(metadata.clone())
                .maybe_song(song.clone())
                .maybe_album(album.clone())
                .maybe_artists(artists.clone())
                .maybe_genres(genres.clone())
                .maybe_picture(picture.clone())
                .call()
                .metadata;

            file.clear()
                .dump_metadata(&self.config.parsing, metadata.clone())
                .save_to(&mut asset, self.mock.config.lofty_write);

            asset.flush().unwrap();
            asset.set_position(0);
            let data = asset.into_inner();

            let path = path.to_path();
            self.to_impl().write(path, &data).await;

            let relative_path = self.relativize(&path).to_path_buf();
            self.filesystem.shift_remove(&relative_path);
            self.filesystem.insert(
                relative_path.clone(),
                Information {
                    information: audio::Information {
                        metadata,
                        property: audio::Property::default(format),
                        file: file::Property::new(format, &data).unwrap(),
                    },
                    relative_path: relative_path.to_string().into(),
                },
            );
        }

        if scan {
            self.scan().run().await.unwrap();
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
            self.scan().run().await.unwrap();
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

    pub fn scan(&self) -> scanner::Scanner<'_, '_, '_> {
        scanner::Scanner::new_orm(
            self.mock.database(),
            self.mock.filesystem(),
            self.config.clone(),
            music_folders::MusicFolder {
                id: self.music_folder.id,
                data: music_folders::Data {
                    path: Cow::Borrowed(self.music_folder.data.path.as_ref()),
                    ty: self.music_folder.data.ty,
                },
            },
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
    ) -> IndexMap<Utf8TypedPathBuf, Information<'static, 'static>> {
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
    use super::*;
    use crate::orm::id3::duration::Trait;
    use crate::Error;

    impl Trait for IndexMap<Uuid, Information<'static, 'static>> {
        fn duration(&self) -> Result<u32, Error> {
            self.values()
                .map(|information| information.information.property.duration)
                .sum::<f32>()
                .duration()
        }
    }
}
