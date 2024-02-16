use crate::database::EncryptionKey;
use crate::Database;
use crate::DatabasePool;

use axum::extract::State;
use concat_string::concat_string;
use diesel_async::{AsyncConnection, AsyncPgConnection};
use url::Url;
use uuid::Uuid;

pub struct TemporaryDatabase {
    name: String,
    root_url: String,
    database: Database,
}

impl TemporaryDatabase {
    async fn new(url: String) -> Self {
        let name = Uuid::new_v4().to_string();
        let mut new_url = Url::parse(&url).expect("can not parse database url");
        new_url.set_path(&name);

        let mut root_conn = AsyncPgConnection::establish(&url)
            .await
            .expect("can not connect to the database");
        diesel_async::RunQueryDsl::execute(
            diesel::sql_query(concat_string!("CREATE DATABASE \"", name, "\";")),
            &mut root_conn,
        )
        .await
        .expect("can not create new database");

        Self {
            name,
            root_url: url,
            database: Database::new(new_url.as_str(), rand::random()).await,
        }
    }

    pub async fn new_from_env() -> Self {
        Self::new(
            std::env::var("DATABASE_URL").expect("please set `DATABASE_URL` environment variable"),
        )
        .await
    }

    pub fn database(&self) -> &Database {
        &self.database
    }

    pub fn state(&self) -> State<Database> {
        State(self.database.clone())
    }

    pub fn get_pool(&self) -> &DatabasePool {
        &self.database().pool
    }

    pub fn get_key(&self) -> &EncryptionKey {
        &self.database().key
    }
}

#[cfg(not(target_env = "musl"))]
impl Drop for TemporaryDatabase {
    fn drop(&mut self) {
        use diesel::{pg::PgConnection, Connection};

        let raw_statement =
            concat_string!("DROP DATABASE IF EXISTS \"", &self.name, "\" WITH (FORCE);");

        let mut conn = match PgConnection::establish(&self.root_url) {
            Ok(conn) => conn,
            Err(e) => {
                println!("{}", e);
                println!(
                    "can not drop database, please drop the database manually with '{}'",
                    &raw_statement
                );
                return;
            }
        };

        if let Err(e) = diesel::RunQueryDsl::execute(diesel::sql_query(&raw_statement), &mut conn) {
            println!("{}", e);
            println!(
                "can not drop database, please drop the database manually with '{}'",
                &raw_statement
            )
        }
    }
}
