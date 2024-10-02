#![allow(clippy::struct_field_names)]

use std::borrow::Cow;
use std::io::{Cursor, Write};

use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use fake::{Fake, Faker};
use futures_lite::{stream, StreamExt};
use indexmap::IndexMap;
use typed_path::{Utf8TypedPath, Utf8TypedPathBuf};
use uuid::Uuid;

use crate::file::{self, audio, File};
use crate::filesystem::Trait as _;
use crate::orm::{albums, music_folders, songs};
use crate::scan::scanner;
use crate::test::assets;
use crate::test::file::audio::dump::Metadata as _;
use crate::test::filesystem::{self, Trait as _};

pub struct Mock<'a> {
    mock: &'a super::Mock,
    music_folder: music_folders::MusicFolder<'static>,
    pub audio: IndexMap<Utf8TypedPathBuf, audio::Information<'static>>,
}

#[bon::bon]
impl<'a> Mock<'a> {
    pub async fn new(mock: &'a super::Mock, index: usize) -> Self {
        Self {
            mock,
            audio: IndexMap::new(),
            music_folder: music_folders::table
                .select(music_folders::MusicFolder::as_select())
                .order_by(music_folders::created_at)
                .offset(index.try_into().unwrap())
                .first(&mut mock.get().await)
                .await
                .unwrap(),
        }
    }

    pub fn config(&self) -> &super::Config {
        &self.mock.config
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

    pub fn absolutize(&self, path: impl AsRef<str>) -> Utf8TypedPathBuf {
        let path = self.path_str(&path);
        let music_folder_path = self.path();

        if path.is_absolute() {
            if path.starts_with(&music_folder_path) {
                path.to_path_buf()
            } else {
                panic!("Path {path} does not start with music folder path {music_folder_path}")
            }
        } else {
            music_folder_path.join(path)
        }
    }

    pub fn relativize<'b>(&self, path: &'b Utf8TypedPath<'b>) -> Utf8TypedPath<'b> {
        if path.is_absolute() { path.strip_prefix(self.path()).unwrap() } else { path.clone() }
    }

    pub fn to_impl(&self) -> filesystem::Impl<'_> {
        self.mock.to_impl(self.music_folder.data.ty.into())
    }

    #[builder]
    pub async fn add_audio(
        &mut self,
        path: Option<impl AsRef<str>>,
        #[builder(default = (0..3).fake::<usize>())] depth: usize,
        #[builder(default = Faker.fake::<audio::Format>())] format: audio::Format,
        metadata: Option<audio::Metadata<'static>>,
        song: Option<audio::Song<'static>>,
        album: Option<audio::Album<'static>>,
        artists: Option<audio::Artists<'static>>,
        genres: Option<audio::Genres<'static>>,
        #[builder(default = 1)] n_song: usize,
        #[builder(default = true)] scan: bool,
    ) -> &mut Self {
        for _ in 0..n_song {
            let path = if n_song == 1
                && let Some(ref path) = path
            {
                self.absolutize(path)
            } else {
                self.absolutize(self.to_impl().fake_path(depth))
            }
            .with_extension(format.as_ref());

            let data = tokio::fs::read(assets::path(format).as_str()).await.unwrap();
            let mut asset = Cursor::new(data.clone());
            let mut file =
                File::new(data, format).unwrap().audio(self.config().lofty_parse).unwrap();
            asset.set_position(0);

            let metadata = metadata.clone().unwrap_or_else(|| audio::Metadata {
                song: song.clone().unwrap_or_else(|| Faker.fake()),
                album: album.clone().unwrap_or_else(|| Faker.fake()),
                artists: artists.clone().unwrap_or_else(|| Faker.fake()),
                genres: genres.clone().unwrap_or_else(|| Faker.fake()),
            });

            file.clear()
                .dump_metadata(&self.config().parsing, metadata.clone())
                .save_to(&mut asset, self.config().lofty_write);

            asset.flush().unwrap();
            asset.set_position(0);
            let data = asset.into_inner();

            self.to_impl().write(path.to_path(), &data).await;

            let relative_path = self.relativize(&path.to_path()).to_path_buf();
            self.audio.shift_remove(&relative_path);
            self.audio.insert(
                relative_path,
                audio::Information {
                    metadata,
                    property: audio::Property::default(format),
                    file: file::Property::new(&data, format).unwrap(),
                },
            );
        }

        if scan {
            self.scan().run().await.unwrap();
        }

        self
    }

    #[builder]
    pub async fn remove_audio(
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
            self.audio.shift_remove(&relative_path);
        } else if let Some((relative_path, _)) = self.audio.shift_remove_index(index) {
            self.to_impl().delete(self.absolutize(relative_path).to_path()).await;
        }

        if scan {
            self.scan().run().await.unwrap();
        }

        self
    }

    pub async fn file(&self, path: Utf8TypedPath<'_>, format: audio::Format) -> audio::File {
        let path = self.absolutize(path).with_extension(format.as_ref());
        File::new(self.to_impl().read(path.to_path()).await.unwrap(), format)
            .unwrap()
            .audio(self.config().lofty_parse)
            .unwrap()
    }

    pub fn scan(&self) -> scanner::Scanner<'_, '_, '_> {
        scanner::Scanner::new_orm(
            self.mock.database(),
            self.mock.filesystem(),
            scanner::Config {
                lofty: self.config().lofty_parse,
                scan: self.config().filesystem.scan,
                parsing: self.config().parsing.clone(),
                index: self.config().index.clone(),
            },
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

    pub async fn query(
        &self,
        absolutize: bool,
    ) -> IndexMap<Utf8TypedPathBuf, audio::Information<'static>> {
        let song_ids = albums::table
            .inner_join(songs::table)
            .inner_join(music_folders::table)
            .filter(music_folders::id.eq(self.music_folder.id))
            .select(songs::id)
            .get_results(&mut self.mock.get().await)
            .await
            .unwrap();
        stream::iter(song_ids)
            .then(async |id| {
                let (path, information) = audio::Information::query_path(self.mock, id).await;
                let path = if absolutize { self.absolutize(path) } else { self.path_string(path) };
                (path, information)
            })
            .collect()
            .await
    }
}
