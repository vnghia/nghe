use diesel::dsl::{auto_type, exists};
use diesel::{ExpressionMethods, QueryDsl};
use uuid::Uuid;

use super::{albums, user_music_folder_permissions};

#[auto_type]
pub fn query(user_id: Uuid) -> _ {
    exists(
        user_music_folder_permissions::table
            .filter(user_music_folder_permissions::user_id.eq(user_id))
            .filter(user_music_folder_permissions::music_folder_id.eq(albums::music_folder_id)),
    )
}
