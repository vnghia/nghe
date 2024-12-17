use diesel::prelude::*;
use diesel_derives::AsChangeset;
use uuid::Uuid;

pub use crate::schema::star_songs::{self, *};

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = star_songs, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Upsert {
    pub user_id: Uuid,
    pub song_id: Uuid,
}

mod upsert {
    use diesel_async::RunQueryDsl;
    use uuid::Uuid;

    use super::{Upsert, star_songs};
    use crate::Error;
    use crate::database::Database;

    impl Upsert {
        pub async fn upserts(
            database: &Database,
            user_id: Uuid,
            song_ids: &[Uuid],
        ) -> Result<(), Error> {
            diesel::insert_into(star_songs::table)
                .values(
                    song_ids
                        .iter()
                        .copied()
                        .map(|song_id| Self { user_id, song_id })
                        .collect::<Vec<_>>(),
                )
                .on_conflict_do_nothing()
                .execute(&mut database.get().await?)
                .await?;
            Ok(())
        }
    }
}
