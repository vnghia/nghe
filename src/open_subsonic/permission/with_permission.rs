use diesel::dsl::{exists, Eq, Filter};
use diesel::{ExpressionMethods, QueryDsl};
use uuid::Uuid;

use crate::models::*;

pub type WithPermission = exists<
    Filter<
        Filter<
            user_music_folder_permissions::table,
            Eq<user_music_folder_permissions::user_id, Uuid>,
        >,
        Eq<user_music_folder_permissions::music_folder_id, songs::music_folder_id>,
    >,
>;

pub fn with_permission(user_id: Uuid) -> WithPermission {
    exists(
        user_music_folder_permissions::table
            .filter(user_music_folder_permissions::user_id.eq(user_id))
            .filter(user_music_folder_permissions::music_folder_id.eq(songs::music_folder_id)),
    )
}
