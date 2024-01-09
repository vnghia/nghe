use crate::Migrator;

use concat_string::concat_string;
use sea_orm::{
    ActiveModelTrait, ConnectionTrait, Database, DatabaseConnection, DbErr, EntityTrait, Statement,
};
use sea_orm_migration::prelude::*;
use url::Url;
use uuid::Uuid;

#[derive(Debug, Clone, Default)]
pub struct TemporaryDatabase {
    name: String,
    conn: DatabaseConnection,
    old_conn: DatabaseConnection,
}

impl TemporaryDatabase {
    async fn new(url: &String) -> Self {
        let name = Uuid::new_v4().to_string();
        let mut new_url = Url::parse(url).expect("can not parse database url");
        new_url.set_path(&name);

        let old_conn = Database::connect(url)
            .await
            .expect("can not connect to the database");

        old_conn
            .execute(Statement::from_string(
                old_conn.get_database_backend(),
                concat_string!("CREATE DATABASE \"", name, "\";"),
            ))
            .await
            .expect("can not create new database");

        let conn = Database::connect(new_url)
            .await
            .expect("can not connect to the new database");
        println!("create new database with name \"{}\"", name);

        Migrator::up(&conn, None)
            .await
            .expect("can not run pending migration(s)");

        Self {
            name,
            conn,
            old_conn,
        }
    }

    pub async fn new_from_env() -> Self {
        Self::new(
            &std::env::var("DATABASE_URL").expect("please set `DATABASE_URL` environment variable"),
        )
        .await
    }

    pub fn get_conn(&self) -> &DatabaseConnection {
        &self.conn
    }

    pub async fn insert<A: ActiveModelTrait>(&self, model: A) -> &Self {
        A::Entity::insert(model)
            .exec(&self.conn)
            .await
            .expect(&concat_string!(
                "can not insert into database \"",
                &self.name,
                "\""
            ));
        &self
    }

    // TODO: implement actual async drop
    pub async fn async_drop(&self) {
        let raw_statement =
            concat_string!("DROP DATABASE IF EXISTS \"", &self.name, "\" WITH (FORCE);");
        match self
            .old_conn
            .execute(Statement::from_string(
                self.old_conn.get_database_backend(),
                &raw_statement,
            ))
            .await
        {
            Err(e) => {
                println!("{}", e);
                println!(
                    "can not drop database, please drop the database manually with '{}'",
                    &raw_statement
                )
            }
            _ => (),
        }
    }
}

#[tokio::test]
async fn test_temporary_database() {
    let db = TemporaryDatabase::new_from_env().await;
    assert!(db.get_conn().ping().await.is_ok());
    db.async_drop().await;
    assert!(matches!(db.get_conn().ping().await, Err(DbErr::Conn(_))));
}
