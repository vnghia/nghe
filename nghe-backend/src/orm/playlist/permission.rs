use diesel::dsl::{exists, select};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::database::Database;
use crate::{error, Error};

pub async fn check_write(
    database: &Database,
    playlist_id: Uuid,
    user_id: Uuid,
    owner: bool,
) -> Result<(), Error> {
    let exist = if owner {
        select(exists(query::owner(playlist_id, user_id)))
            .get_result(&mut database.get().await?)
            .await?
    } else {
        select(exists(query::write(playlist_id, user_id)))
            .get_result(&mut database.get().await?)
            .await?
    };
    if exist { Ok(()) } else { error::Kind::NotFound.into() }
}

pub mod query {
    use diesel::dsl::auto_type;
    use diesel::{ExpressionMethods, QueryDsl};

    use super::*;
    use crate::orm::playlists_users;

    #[auto_type]
    pub fn read(playlist_id: Uuid, user_id: Uuid) -> _ {
        playlists_users::table
            .filter(playlists_users::playlist_id.eq(playlist_id))
            .filter(playlists_users::user_id.eq(user_id))
    }

    #[auto_type]
    pub fn write(playlist_id: Uuid, user_id: Uuid) -> _ {
        let read: read = read(playlist_id, user_id);
        read.filter(playlists_users::write)
    }

    #[auto_type]
    pub fn owner(playlist_id: Uuid, user_id: Uuid) -> _ {
        let read: read = read(playlist_id, user_id);
        read.filter(playlists_users::owner)
    }
}
