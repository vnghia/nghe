use diesel::dsl::count_distinct;
use diesel::prelude::*;
use nghe_api::id3;
use o2o::o2o;

use crate::orm::{albums, genres, songs};
use crate::Error;

#[derive(Debug, Queryable, Selectable, o2o)]
#[owned_try_into(id3::genre::WithCount, Error)]
#[diesel(table_name = genres, check_for_backend(crate::orm::Type))]
pub struct WithCount {
    pub value: String,
    #[into(~.try_into()?)]
    #[diesel(select_expression = count_distinct(songs::id))]
    pub song_count: i64,
    #[into(~.try_into()?)]
    #[diesel(select_expression = count_distinct(albums::id))]
    pub album_count: i64,
}

pub mod query {
    use diesel::dsl::{auto_type, AsSelect};
    use uuid::Uuid;

    use super::*;
    use crate::orm::{permission, songs_genres};

    #[auto_type]
    pub fn with_user_id(user_id: Uuid) -> _ {
        let with_count: AsSelect<WithCount, crate::orm::Type> = WithCount::as_select();
        let permission: permission::with_album = permission::with_album(user_id);
        genres::table
            .inner_join(songs_genres::table)
            .inner_join(songs::table.on(songs::id.eq(songs_genres::song_id)))
            .inner_join(albums::table.on(albums::id.eq(songs::album_id)))
            .filter(permission)
            .group_by(genres::id)
            .order_by(genres::value)
            .select(with_count)
    }
}
