use diesel::prelude::*;
use uuid::Uuid;

pub use crate::schema::playlists_users::{self, *};

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = playlists_users, check_for_backend(crate::orm::Type))]
pub struct Upsert {
    pub playlist_id: Uuid,
    pub user_id: Uuid,
    pub write: bool,
}

mod upsert {
    use diesel::ExpressionMethods;
    use diesel_async::RunQueryDsl;
    use uuid::Uuid;

    use super::{Upsert, playlists_users};
    use crate::Error;
    use crate::database::Database;

    impl Upsert {
        pub async fn insert_owner(
            database: &Database,
            playlist_id: Uuid,
            user_id: Uuid,
        ) -> Result<(), Error> {
            diesel::insert_into(playlists_users::table)
                .values((
                    Upsert { playlist_id, user_id, write: true },
                    playlists_users::owner.eq(true),
                ))
                .execute(&mut database.get().await?)
                .await?;
            Ok(())
        }
    }
}
