#![allow(clippy::struct_field_names)]

use std::io::{Cursor, Write};

use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use fake::{Fake, Faker};
use futures_lite::{stream, StreamExt};
use indexmap::IndexMap;
use typed_path::{Utf8TypedPath, Utf8TypedPathBuf};

use crate::file::{self, audio, File};
use crate::filesystem::Trait as _;
use crate::orm::{albums, music_folders, songs};
use crate::scan::scanner;
use crate::test::assets;
use crate::test::file::audio::dump::Metadata as _;
use crate::test::filesystem::{self, Trait as _};

pub struct Mock<'a> {
    mock: &'a super::Mock,
    pub audio: IndexMap<Utf8TypedPathBuf, audio::Information<'static>>,
    pub music_folder: music_folders::MusicFolder<'static>,
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

    pub fn absolutize(&self, path: impl AsRef<str>) -> Utf8TypedPathBuf {
        self.to_impl().path().from_str(&self.music_folder.data.path).join(path)
    }

    pub fn relativize<'b>(&self, path: &'b Utf8TypedPath<'b>) -> Utf8TypedPath<'b> {
        path.strip_prefix(&self.music_folder.data.path).unwrap()
    }

    pub fn to_impl(&self) -> filesystem::Impl<'_> {
        self.mock.to_impl(self.music_folder.data.ty.into())
    }

    #[builder]
    pub async fn add_audio(
        &mut self,
        path: Option<Utf8TypedPath<'_>>,
        #[builder(default = (0..3).fake::<usize>())] depth: usize,
        #[builder(default = Faker.fake::<audio::Format>())] format: audio::Format,
        metadata: Option<audio::Metadata<'static>>,
        song: Option<audio::Song<'static>>,
        album: Option<audio::NameDateMbz<'static>>,
        artists: Option<audio::Artists<'static>>,
        genres: Option<audio::Genres<'static>>,
        #[builder(default = 1)] n_song: usize,
        #[builder(default = true)] scan: bool,
    ) -> &Self {
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
                File::new(data, format).unwrap().audio(self.mock.config.lofty_parse).unwrap();
            asset.set_position(0);

            let metadata = metadata.clone().unwrap_or_else(|| audio::Metadata {
                song: song.clone().unwrap_or_else(|| Faker.fake()),
                album: album.clone().unwrap_or_else(|| Faker.fake()),
                artists: artists.clone().unwrap_or_else(|| Faker.fake()),
                genres: genres.clone().unwrap_or_else(|| Faker.fake()),
            });

            file.clear()
                .dump_metadata(&self.mock.config.parsing, metadata.clone())
                .save_to(&mut asset, self.mock.config.lofty_write);

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

    pub async fn file(&self, path: Utf8TypedPath<'_>, format: audio::Format) -> audio::File {
        let path = self.absolutize(path).with_extension(format.as_ref());
        File::new(self.to_impl().read(path.to_path()).await.unwrap(), format)
            .unwrap()
            .audio(self.mock.config.lofty_parse)
            .unwrap()
    }

    pub fn scan(&self) -> scanner::Scanner<'_, '_> {
        scanner::Scanner::new_orm(
            self.mock.database(),
            self.mock.filesystem(),
            scanner::Config {
                lofty: self.mock.config.lofty_parse,
                scan: self.mock.config.filesystem.scan,
                parsing: self.mock.config.parsing.clone(),
                index: self.mock.config.index.clone(),
            },
            self.music_folder.clone(),
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
                let path = if absolutize {
                    self.absolutize(path)
                } else {
                    self.to_impl().path().from_string(path)
                };
                (path, information)
            })
            .collect()
            .await
    }
}
