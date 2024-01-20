use diesel::Connection;
use diesel_async::async_connection_wrapper::AsyncConnectionWrapper;
use diesel_async::AsyncPgConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub async fn run_pending_migrations(database_url: &str) {
    let database_url = database_url.to_string();
    tokio::task::spawn_blocking(move || {
        let mut async_wrapper =
            AsyncConnectionWrapper::<AsyncPgConnection>::establish(&database_url)
                .expect("can not connect to the database");
        async_wrapper.run_pending_migrations(MIGRATIONS).unwrap();
    })
    .await
    .expect("can not run pending migration(s)");
}
