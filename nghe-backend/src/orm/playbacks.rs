use diesel::prelude::*;
use time::OffsetDateTime;
use uuid::Uuid;

pub use crate::schema::playbacks::{self, *};

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = playbacks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Scrobble {
    pub user_id: Uuid,
    pub song_id: Uuid,
    pub updated_at: OffsetDateTime,
}

mod upsert {
    use diesel::ExpressionMethods;
    use diesel::upsert::excluded;
    use diesel_async::RunQueryDsl;

    use super::{Scrobble, playbacks};
    use crate::Error;
    use crate::database::Database;

    impl Scrobble {
        pub async fn upsert(database: &Database, values: &[Self]) -> Result<(), Error> {
            diesel::insert_into(playbacks::table)
                .values(values)
                .on_conflict((playbacks::user_id, playbacks::song_id))
                .do_update()
                .set((
                    playbacks::count.eq(playbacks::count + 1),
                    playbacks::updated_at.eq(excluded(playbacks::updated_at)),
                ))
                .execute(&mut database.get().await?)
                .await?;
            Ok(())
        }
    }
}
