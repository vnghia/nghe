use crate::models::*;
use crate::{DatabasePool, OSResult};

use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
use itertools::Itertools;
use std::borrow::Cow;
use uuid::Uuid;

pub async fn upsert_album<'a>(pool: &DatabasePool, name: Cow<'a, str>) -> OSResult<Uuid> {
    Ok(diesel::insert_into(albums::table)
        .values(&albums::NewAlbum { name })
        .on_conflict(albums::name)
        .do_update()
        .set(albums::scanned_at.eq(time::OffsetDateTime::now_utc()))
        .returning(albums::id)
        .get_result(&mut pool.get().await?)
        .await?)
}

pub async fn upsert_album_artists(
    pool: &DatabasePool,
    album_id: Uuid,
    artist_ids: &[Uuid],
) -> OSResult<()> {
    diesel::insert_into(albums_artists::table)
        .values(
            artist_ids
                .iter()
                .cloned()
                .map(|artist_id| albums_artists::NewAlbumArtist {
                    album_id,
                    artist_id,
                    upserted_at: time::OffsetDateTime::now_utc(),
                })
                .collect_vec(),
        )
        .on_conflict_do_nothing()
        .execute(&mut pool.get().await?)
        .await?;
    Ok(())
}
