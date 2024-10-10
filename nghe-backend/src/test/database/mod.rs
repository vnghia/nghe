use concat_string::concat_string;
use diesel_async::{AsyncConnection, AsyncPgConnection};
use url::Url;
use uuid::Uuid;

use crate::database::Database;
use crate::{config, migration};

pub struct Mock {
    name: String,
    url: String,
    database: Database,
}

impl Mock {
    pub async fn new() -> Self {
        let url = std::env::var("DATABASE_URL").unwrap();

        let _ = tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "info".into()),
            )
            .with_test_writer()
            .try_init();

        let name = Uuid::new_v4().to_string();
        let mut mock_url = Url::parse(&url).unwrap();
        mock_url.set_path(&name);

        let mut root_conn = AsyncPgConnection::establish(&url).await.unwrap();
        diesel_async::RunQueryDsl::execute(
            diesel::sql_query(concat_string!("CREATE DATABASE \"", name, "\";")),
            &mut root_conn,
        )
        .await
        .unwrap();

        let mock_url = mock_url.to_string();
        migration::run(&mock_url).await;

        Self {
            name,
            url,
            database: Database::new(&config::Database { url: mock_url, key: rand::random() }),
        }
    }

    pub fn database(&self) -> &Database {
        &self.database
    }
}

#[cfg(not(any(target_env = "musl", all(target_arch = "aarch64", target_os = "linux"))))]
impl Drop for Mock {
    fn drop(&mut self) {
        use diesel::pg::PgConnection;
        use diesel::Connection;

        let raw_statement =
            concat_string!("DROP DATABASE IF EXISTS \"", &self.name, "\" WITH (FORCE);");
        if let Err::<_, color_eyre::Report>(e) = try {
            let mut conn = PgConnection::establish(&self.url)?;
            diesel::RunQueryDsl::execute(diesel::sql_query(&raw_statement), &mut conn)?;
        } {
            println!("Could not drop temporary database because of {:?}", &e);
            println!("Please drop the database manually with '{}'", &raw_statement);
        }
    }
}
