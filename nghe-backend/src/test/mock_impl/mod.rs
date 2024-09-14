mod music_folder;
mod user;

use derivative::Derivative;
use diesel_async::pooled_connection::deadpool;
use diesel_async::AsyncPgConnection;
use fake::{Fake, Faker};
use lofty::config::{ParseOptions, WriteOptions};
use nghe_api::common;
use rstest::fixture;

use super::filesystem::Trait;
use super::{database, filesystem};
use crate::database::Database;
use crate::filesystem::Filesystem;
use crate::orm::users;
use crate::{config, route};

#[derive(Debug, Derivative)]
#[derivative(Default)]
pub struct Config {
    pub filesystem: config::filesystem::Filesystem,
    #[derivative(Default(value = "config::Parsing::test()"))]
    pub parsing: config::Parsing,

    pub lofty_parse: ParseOptions,
    pub lofty_write: WriteOptions,
}

pub struct Mock {
    pub config: Config,

    pub database: database::Mock,
    pub filesystem: filesystem::Mock,
}

#[bon::bon]
impl Mock {
    async fn new(prefix: Option<&str>, config: Config) -> Self {
        let database = database::Mock::new().await;
        let filesystem = filesystem::Mock::new(prefix, &config).await;

        Self { config, database, filesystem }
    }

    pub fn state(&self) -> &Database {
        self.database()
    }

    pub fn database(&self) -> &Database {
        self.database.state()
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

    pub fn filesystem(&self) -> &Filesystem {
        self.filesystem.state()
    }

    pub fn to_impl(&self, filesystem_type: common::filesystem::Type) -> filesystem::Impl<'_> {
        self.filesystem.to_impl(filesystem_type)
    }

    #[builder]
    pub async fn add_music_folder(
        &self,
        #[builder(default = Faker.fake::<common::filesystem::Type>())]
        filesystem_type: common::filesystem::Type,
        #[builder(default = true)] allow: bool,
    ) -> &Self {
        let filesystem = self.to_impl(filesystem_type);
        route::music_folder::add::handler(
            self.database(),
            self.filesystem(),
            route::music_folder::add::Request {
                filesystem_type,
                allow,
                path: filesystem
                    .create_dir(Faker.fake::<String>().as_str().into())
                    .await
                    .into_string(),
                ..Faker.fake()
            },
        )
        .await
        .unwrap();
        self
    }

    pub async fn music_folder(&self, index: usize) -> music_folder::Mock<'_> {
        music_folder::Mock::new(self, index).await
    }
}

#[fixture]
pub async fn mock(
    #[default(1)] n_user: usize,
    #[default(1)] n_music_folder: usize,
    #[default(None)] prefix: Option<&str>,
    #[default(Config::default())] config: Config,
) -> Mock {
    let mock = Mock::new(prefix, config).await;
    for _ in 0..n_user {
        mock.add_user().call().await;
    }
    for _ in 0..n_music_folder {
        mock.add_music_folder()
            .filesystem_type(Faker.fake::<common::filesystem::Type>())
            .call()
            .await;
    }
    mock
}
