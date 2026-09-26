use diesel::prelude::*;
use uuid::Uuid;

pub use crate::schema::playlists_songs::{self, *};

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = playlists_songs, check_for_backend(crate::orm::Type))]
pub struct Upsert {
    pub playlist_id: Uuid,
    pub song_id: Uuid,
}

mod upsert {
    use diesel::ExpressionMethods;
    use diesel_async::RunQueryDsl;
    use uuid::Uuid;

    use super::{Upsert, playlists_songs};
    use crate::Error;
    use crate::database::Database;

    impl Upsert {
        pub async fn upsert(&self, database: &Database) -> Result<(), Error> {
            diesel::insert_into(playlists_songs::table)
                .values((self, playlists_songs::created_at.eq(crate::time::now().await)))
                .on_conflict_do_nothing()
                .execute(&mut database.get().await?)
                .await?;
            Ok(())
        }

        pub async fn upserts(
            database: &Database,
            playlist_id: Uuid,
            song_ids: &[Uuid],
        ) -> Result<(), Error> {
            // Upserting one by one to avoid two songs having too close `created_at`.
            for song_id in song_ids.iter().copied() {
                Self { playlist_id, song_id }.upsert(database).await?;
            }
            Ok(())
        }
    }
}
