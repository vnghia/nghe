use std::borrow::Cow;

use anyhow::Result;
use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
use itertools::Itertools;
use uuid::Uuid;

use crate::models::*;
use crate::DatabasePool;

pub async fn upsert_album<'a>(pool: &DatabasePool, name: Cow<'a, str>) -> Result<Uuid> {
    diesel::insert_into(albums::table)
        .values(&albums::NewAlbum { name })
        .on_conflict(albums::name)
        .do_update()
        .set(albums::scanned_at.eq(time::OffsetDateTime::now_utc()))
        .returning(albums::id)
        .get_result(&mut pool.get().await?)
        .await
        .map_err(anyhow::Error::from)
}

pub async fn upsert_song_album_artists(
    pool: &DatabasePool,
    song_id: Uuid,
    album_artist_ids: &[Uuid],
) -> Result<()> {
    diesel::insert_into(songs_album_artists::table)
        .values(
            album_artist_ids
                .iter()
                .copied()
                .map(|album_artist_id| songs_album_artists::NewSongAlbumArtist {
                    song_id,
                    album_artist_id,
                })
                .collect_vec(),
        )
        .on_conflict((songs_album_artists::song_id, songs_album_artists::album_artist_id))
        .do_update()
        .set(songs_album_artists::upserted_at.eq(time::OffsetDateTime::now_utc()))
        .execute(&mut pool.get().await?)
        .await?;
    Ok(())
}
