mod user;

use diesel_async::pooled_connection::deadpool;
use diesel_async::AsyncPgConnection;
use fake::{Fake, Faker};

use super::database;
use crate::app;
use crate::orm::users;

pub struct Mock {
    pub database: database::Mock,
}

#[bon::bon]
impl Mock {
    pub async fn new() -> Self {
        let database = database::Mock::new().await;

        Self { database }
    }

    pub fn database(&self) -> &app::state::Database {
        &self.database.state
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
        let users::Role { admin, stream, download, share } = role;
        app::user::create::handler(
            self.database(),
            app::user::create::Request { admin, stream, download, share, allow, ..Faker.fake() },
        )
        .await
        .unwrap();
        self
    }

    pub async fn user(&self, index: usize) -> user::Mock {
        user::Mock::new(self, index).await
    }
}
