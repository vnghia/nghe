use diesel::prelude::*;
use diesel_derives::AsChangeset;
use uuid::Uuid;

pub use crate::schema::star_artists::{self, *};

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = star_artists, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Upsert {
    pub user_id: Uuid,
    pub artist_id: Uuid,
}

mod upsert {
    use diesel_async::RunQueryDsl;
    use uuid::Uuid;

    use super::{Upsert, star_artists};
    use crate::Error;
    use crate::database::Database;

    impl Upsert {
        pub async fn upserts(
            database: &Database,
            user_id: Uuid,
            artist_ids: &[Uuid],
        ) -> Result<(), Error> {
            diesel::insert_into(star_artists::table)
                .values(
                    artist_ids
                        .iter()
                        .copied()
                        .map(|artist_id| Self { user_id, artist_id })
                        .collect::<Vec<_>>(),
                )
                .on_conflict_do_nothing()
                .execute(&mut database.get().await?)
                .await?;
            Ok(())
        }
    }
}
