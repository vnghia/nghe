use std::path::{Path, PathBuf};

use axum::extract::State;
use concat_string::concat_string;
use diesel_async::{AsyncConnection, AsyncPgConnection};
use url::Url;
use uuid::Uuid;

use crate::database::EncryptionKey;
use crate::models::*;
use crate::utils::song::test::SongTag;
use crate::utils::song::SongLyric;
use crate::{Database, DatabasePool};

#[derive(Debug)]
pub struct SongDbInformation {
    pub tag: SongTag,
    pub lrc: Option<SongLyric>,
    pub song_id: Uuid,
    pub album_id: Uuid,
    pub artist_ids: Vec<Uuid>,
    pub album_artist_ids: Vec<Uuid>,
    // Filesystem property
    pub music_folder: music_folders::MusicFolder,
    pub relative_path: String,
    pub file_hash: u64,
    pub file_size: u32,
}

impl SongDbInformation {
    pub fn absolute_path(&self) -> PathBuf {
        Path::new(&self.music_folder.path).join(&self.relative_path)
    }
}

pub struct TemporaryDb {
    name: String,
    root_url: String,
    database: Database,
}

impl TemporaryDb {
    async fn new(url: String) -> Self {
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();

        let name = Uuid::new_v4().to_string();
        let mut new_url = Url::parse(&url).expect("can not parse database url");
        new_url.set_path(&name);

        let mut root_conn =
            AsyncPgConnection::establish(&url).await.expect("can not connect to the database");
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

    pub fn pool(&self) -> &DatabasePool {
        &self.database().pool
    }

    pub fn key(&self) -> &EncryptionKey {
        &self.database().key
    }
}

#[cfg(not(any(target_env = "musl", all(target_arch = "aarch64", target_os = "linux"))))]
impl Drop for TemporaryDb {
    fn drop(&mut self) {
        use diesel::pg::PgConnection;
        use diesel::Connection;

        let raw_statement =
            concat_string!("DROP DATABASE IF EXISTS \"", &self.name, "\" WITH (FORCE);");
        if let Err::<_, anyhow::Error>(e) = try {
            let mut conn = PgConnection::establish(&self.root_url)?;
            diesel::RunQueryDsl::execute(diesel::sql_query(&raw_statement), &mut conn)?;
        } {
            println!("could not drop temporary database because of {:?}", &e);
            println!("please drop the database manually with '{}'", &raw_statement);
        }
    }
}
