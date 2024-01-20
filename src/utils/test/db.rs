use crate::migration;
use crate::DatabasePool;

use concat_string::concat_string;
use diesel_async::{
    pooled_connection::AsyncDieselConnectionManager, AsyncConnection, AsyncPgConnection,
};
use url::Url;
use uuid::Uuid;

pub struct TemporaryDatabase {
    name: String,
    pool: DatabasePool,
    root_url: String,
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

        let pool = DatabasePool::builder(AsyncDieselConnectionManager::<
            diesel_async::AsyncPgConnection,
        >::new(new_url.as_str()))
        .build()
        .expect("can not connect to the new database");
        println!("create new database with name \"{}\"", name);

        migration::run_pending_migrations(new_url.as_str()).await;

        Self {
            name,
            pool,
            root_url: url,
        }
    }

    pub async fn new_from_env() -> Self {
        Self::new(
            std::env::var("DATABASE_URL").expect("please set `DATABASE_URL` environment variable"),
        )
        .await
    }

    pub fn get_pool(&self) -> &DatabasePool {
        &self.pool
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
