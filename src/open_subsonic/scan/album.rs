use crate::models::*;
use crate::{DatabasePool, OSResult};

use diesel::SelectableHelper;
use diesel_async::RunQueryDsl;
use std::borrow::Cow;

pub async fn upsert_album<'a>(pool: &DatabasePool, name: Cow<'a, str>) -> OSResult<albums::Album> {
    Ok(diesel::insert_into(albums::table)
        .values(&albums::NewAlbum { name })
        .on_conflict_do_nothing()
        .returning(albums::Album::as_returning())
        .get_result(&mut pool.get().await?)
        .await?)
}
