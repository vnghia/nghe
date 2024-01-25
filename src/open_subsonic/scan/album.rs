use crate::models::*;
use crate::{DatabasePool, OSResult};

use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
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
