mod user;

use diesel_async::pooled_connection::deadpool;
use diesel_async::AsyncPgConnection;
use fake::{Fake, Faker};
use nghe_api::music_folder::FilesystemType;

use super::{database, filesystem};
use crate::app;
use crate::orm::users;

pub struct Mock {
    pub database: database::Mock,
    pub filesystem: filesystem::Mock,
}

#[bon::bon]
impl Mock {
    pub async fn new() -> Self {
        let database = database::Mock::new().await;
        let filesystem = filesystem::Mock::new();

        Self { database, filesystem }
    }

    pub fn database(&self) -> &app::state::Database {
        self.database.state()
    }

    pub async fn get(&self) -> deadpool::Object<AsyncPgConnection> {
        self.database().get().await.unwrap()
    }

    #[builder]
    pub async fn add_user(
        self,
        #[builder(default = users::Role {
            admin: false,
            stream: true,
            download: true,
            share: false
        })]
        role: users::Role,
        #[builder(default = true)] allow: bool,
    ) -> Self {
        app::user::create::handler(
            self.database(),
            app::user::create::Request { role: role.into(), allow, ..Faker.fake() },
        )
        .await
        .unwrap();
        self
    }

    pub async fn user(&self, index: usize) -> user::Mock {
        user::Mock::new(self, index).await
    }

    pub fn filesystem(&self) -> &app::state::Filesystem {
        self.filesystem.state()
    }

    pub fn to_impl(&self, filesystem_type: FilesystemType) -> filesystem::MockImpl<'_> {
        self.filesystem.to_impl(filesystem_type)
    }

    #[builder]
    pub async fn add_folder(
        self,
        #[builder(default = Faker.fake::<FilesystemType>())] filesystem_type: FilesystemType,
        #[builder(default = true)] allow: bool,
    ) -> Self {
        app::music_folder::add::handler(
            self.database(),
            self.filesystem(),
            app::music_folder::add::Request { filesystem_type, allow, ..Faker.fake() },
        )
        .await
        .unwrap();
        self
    }
}
