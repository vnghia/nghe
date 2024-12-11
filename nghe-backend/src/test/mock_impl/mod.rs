#![allow(clippy::option_option)]

mod information;
mod music_folder;
mod user;

use diesel_async::pooled_connection::deadpool;
use diesel_async::AsyncPgConnection;
use educe::Educe;
use fake::{Fake, Faker};
pub use information::Mock as Information;
use lofty::config::{ParseOptions, WriteOptions};
use nghe_api::common;
use rstest::fixture;
use typed_path::Utf8PlatformPath;
use uuid::Uuid;

use super::filesystem::Trait;
use super::{database, filesystem};
use crate::database::Database;
use crate::file::audio;
use crate::filesystem::Filesystem;
use crate::integration::Informant;
use crate::orm::users;
use crate::scan::scanner;
use crate::{config, route};

#[derive(Debug, Educe)]
#[educe(Default)]
pub struct Config {
    #[educe(Default(expression = config::filesystem::Filesystem::test()))]
    pub filesystem: config::filesystem::Filesystem,
    #[educe(Default(expression = config::Parsing::test()))]
    pub parsing: config::Parsing,
    pub index: config::Index,
    pub transcode: config::Transcode,
    pub cover_art: config::CoverArt,
    pub integration: config::Integration,

    pub lofty_parse: ParseOptions,
    pub lofty_write: WriteOptions,
}

pub struct Mock {
    pub config: Config,

    pub database: database::Mock,
    pub filesystem: filesystem::Mock,
    pub informant: Informant,
}

impl Config {
    fn with_prefix(self, prefix: impl AsRef<Utf8PlatformPath> + Copy) -> Self {
        Self {
            transcode: self.transcode.with_prefix(prefix),
            cover_art: self.cover_art.with_prefix(prefix),
            ..self
        }
    }

    pub fn scanner(&self) -> scanner::Config {
        scanner::Config {
            lofty: self.lofty_parse,
            scan: self.filesystem.scan,
            parsing: self.parsing.clone(),
            index: self.index.clone(),
            cover_art: self.cover_art.clone(),
        }
    }
}

#[bon::bon]
impl Mock {
    async fn new(prefix: Option<&str>, config: Config) -> Self {
        let database = database::Mock::new().await;
        let filesystem = filesystem::Mock::new(prefix, &config).await;
        let config = config.with_prefix(&filesystem.prefix());
        let informant = Informant::new(config.integration.clone()).await;

        Self { config, database, filesystem, informant }
    }

    pub fn state(&self) -> &Database {
        self.database()
    }

    pub fn database(&self) -> &Database {
        self.database.database()
    }

    pub async fn get(&self) -> deadpool::Object<AsyncPgConnection> {
        self.database().get().await.unwrap()
    }

    #[builder]
    pub async fn add_user(
        &self,
        #[builder(default = users::Role {
            admin: false,
            stream: true,
            download: true,
            share: false
        })]
        role: users::Role,
        #[builder(default = true)] allow: bool,
    ) -> &Self {
        route::user::create::handler(
            self.database(),
            route::user::create::Request { role: role.into(), allow, ..Faker.fake() },
        )
        .await
        .unwrap();
        self
    }

    pub async fn user(&self, index: usize) -> user::Mock<'_> {
        user::Mock::new(self, index).await
    }

    pub async fn user_id(&self, index: usize) -> Uuid {
        self.user(index).await.id()
    }

    pub fn filesystem(&self) -> &Filesystem {
        self.filesystem.filesystem()
    }

    pub fn to_impl(&self, ty: common::filesystem::Type) -> filesystem::Impl<'_> {
        self.filesystem.to_impl(ty)
    }

    #[builder]
    pub async fn add_music_folder(
        &self,
        #[builder(default = Faker.fake::<common::filesystem::Type>())] ty: common::filesystem::Type,
        #[builder(default = true)] allow: bool,
    ) -> Uuid {
        let filesystem = self.to_impl(ty);
        route::music_folder::add::handler(
            self.database(),
            self.filesystem(),
            route::music_folder::add::Request {
                ty,
                allow,
                path: filesystem
                    .create_dir(Faker.fake::<String>().as_str().into())
                    .await
                    .into_string(),
                ..Faker.fake()
            },
        )
        .await
        .unwrap()
        .music_folder_id
    }

    pub async fn music_folder(&self, index: usize) -> music_folder::Mock<'_> {
        music_folder::Mock::new(self, index).await
    }

    pub async fn music_folder_id(&self, index: usize) -> Uuid {
        self.music_folder(index).await.id()
    }

    pub async fn add_audio_artist(
        &self,
        index: usize,
        songs: impl IntoIterator<Item = audio::Artist<'static>>,
        albums: impl IntoIterator<Item = audio::Artist<'static>>,
        compilation: bool,
        n_song: usize,
    ) {
        self.music_folder(index).await.add_audio_artist(songs, albums, compilation, n_song).await;
    }
}

#[fixture]
pub async fn mock(
    #[default(1)] n_user: usize,
    #[default(1)] n_music_folder: usize,
    #[default(None)] prefix: Option<&str>,
    #[default(false)] enable_integration: bool,
) -> Mock {
    let mock = Mock::new(
        prefix,
        Config {
            integration: if enable_integration {
                config::Integration::from_env()
            } else {
                config::Integration::default()
            },
            ..Default::default()
        },
    )
    .await;
    for _ in 0..n_user {
        mock.add_user().call().await;
    }
    for _ in 0..n_music_folder {
        mock.add_music_folder().ty(Faker.fake::<common::filesystem::Type>()).call().await;
    }
    mock.database().upsert_config(&mock.config.index).await.unwrap();

    mock
}
