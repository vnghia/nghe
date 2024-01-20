use crate::migration;
use crate::DbPool;

use concat_string::concat_string;
use diesel::{pg::PgConnection, Connection};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use url::Url;
use uuid::Uuid;

pub struct TemporaryDatabase {
    name: String,
    pool: DbPool,
    old_conn: PgConnection,
}

impl TemporaryDatabase {
    async fn new(url: &str) -> Self {
        let name = Uuid::new_v4().to_string();
        let mut new_url = Url::parse(url).expect("can not parse database url");
        new_url.set_path(&name);

        let mut old_conn = PgConnection::establish(url).expect("can not connect to the database");

        diesel::RunQueryDsl::execute(
            diesel::sql_query(concat_string!("CREATE DATABASE \"", name, "\";")),
            &mut old_conn,
        )
        .expect("can not create new database");

        let pool = DbPool::builder(AsyncDieselConnectionManager::<
            diesel_async::AsyncPgConnection,
        >::new(new_url.as_str()))
        .build()
        .expect("can not connect to the new database");
        println!("create new database with name \"{}\"", name);

        migration::run_pending_migrations(new_url.as_str()).await;

        Self {
            name,
            pool,
            old_conn,
        }
    }

    pub async fn new_from_env() -> Self {
        Self::new(
            &std::env::var("DATABASE_URL").expect("please set `DATABASE_URL` environment variable"),
        )
        .await
    }

    pub fn get_pool(&self) -> &DbPool {
        &self.pool
    }
}

impl Drop for TemporaryDatabase {
    fn drop(&mut self) {
        let raw_statement =
            concat_string!("DROP DATABASE IF EXISTS \"", &self.name, "\" WITH (FORCE);");
        if let Err(e) =
            diesel::RunQueryDsl::execute(diesel::sql_query(&raw_statement), &mut self.old_conn)
        {
            println!("{}", e);
            println!(
                "can not drop database, please drop the database manually with '{}'",
                &raw_statement
            )
        }
    }
}
