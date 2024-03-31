use std::borrow::Cow;

use anyhow::Result;
use diesel::dsl::{exists, not, Eq, Filter};
use diesel::{select, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::models::*;
use crate::{DatabasePool, OSError};

pub fn with_music_folders(
    user_id: Uuid,
) -> exists<
    Filter<
        Filter<
            Filter<
                user_music_folder_permissions::table,
                Eq<user_music_folder_permissions::user_id, Uuid>,
            >,
            Eq<user_music_folder_permissions::music_folder_id, songs::music_folder_id>,
        >,
        user_music_folder_permissions::allow,
    >,
> {
    exists(
        user_music_folder_permissions::table
            .filter(user_music_folder_permissions::user_id.eq(user_id))
            .filter(user_music_folder_permissions::music_folder_id.eq(songs::music_folder_id))
            .filter(user_music_folder_permissions::allow),
    )
}

pub async fn check_user_music_folder_ids<'a>(
    pool: &DatabasePool,
    user_id: &Uuid,
    music_folder_ids: Option<Cow<'a, [Uuid]>>,
) -> Result<Cow<'a, [Uuid]>> {
    if let Some(music_folder_ids) = music_folder_ids {
        if select(not(exists(
            user_music_folder_permissions::table
                .filter(user_music_folder_permissions::user_id.eq(user_id))
                .filter(
                    user_music_folder_permissions::music_folder_id
                        .eq_any(music_folder_ids.as_ref()),
                )
                .filter(not(user_music_folder_permissions::allow)),
        )))
        .first::<bool>(&mut pool.get().await?)
        .await?
        {
            Ok(music_folder_ids)
        } else {
            anyhow::bail!(OSError::Forbidden("access to these music folders".into()))
        }
    } else {
        Ok(user_music_folder_permissions::table
            .select(user_music_folder_permissions::music_folder_id)
            .filter(user_music_folder_permissions::user_id.eq(user_id))
            .filter(user_music_folder_permissions::allow)
            .get_results::<Uuid>(&mut pool.get().await?)
            .await?
            .into())
    }
}
