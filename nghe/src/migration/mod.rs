use diesel::Connection;
use diesel_async::AsyncPgConnection;
use diesel_async::async_connection_wrapper::AsyncConnectionWrapper;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub async fn run(database_url: &str) {
    let database_url = database_url.to_owned();
    tokio::task::spawn_blocking(move || {
        let mut async_wrapper =
            AsyncConnectionWrapper::<AsyncPgConnection>::establish(&database_url)
                .expect("Could not connect to the database");

        for migration in async_wrapper
            .pending_migrations(MIGRATIONS)
            .expect("Could not get pending migration(s)")
        {
            tracing::info!(pending_migration =% migration.name());
            async_wrapper.run_migration(&migration).expect("Could not run migration");
        }
        tracing::info!("migration done");
    })
    .await
    .expect("Could not spawn migration thread");
}
