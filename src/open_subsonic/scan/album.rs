use anyhow::Result;
use diesel::{DecoratableTarget, ExpressionMethods};
use diesel_async::RunQueryDsl;
use itertools::Itertools;
use uuid::Uuid;

use crate::models::*;
use crate::DatabasePool;

pub async fn upsert_album<'a>(
    pool: &DatabasePool,
    album_no_id: albums::NewAlbum<'a>,
) -> Result<Uuid> {
    if album_no_id.mbz_id.is_some() {
        diesel::insert_into(albums::table)
            .values(album_no_id)
            .on_conflict(albums::mbz_id)
            .do_update()
            .set(albums::scanned_at.eq(time::OffsetDateTime::now_utc()))
            .returning(albums::id)
            .get_result::<Uuid>(&mut pool.get().await.map_err(anyhow::Error::from)?)
            .await
    } else {
        diesel::insert_into(albums::table)
            .values(album_no_id)
            .on_conflict((
                albums::name,
                albums::year,
                albums::month,
                albums::day,
                albums::release_year,
                albums::release_month,
                albums::release_day,
                albums::original_release_year,
                albums::original_release_month,
                albums::original_release_day,
            ))
            .filter_target(albums::mbz_id.is_null())
            .do_update()
            .set(albums::scanned_at.eq(time::OffsetDateTime::now_utc()))
            .returning(albums::id)
            .get_result::<Uuid>(&mut pool.get().await.map_err(anyhow::Error::from)?)
            .await
    }
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

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};

    use super::*;
    use crate::utils::test::TemporaryDb;

    #[tokio::test]
    async fn test_upsert_album_mbz_id() {
        let temp_db = TemporaryDb::new_from_env().await;
        let mbz_id = Some(Faker.fake());
        let album_no_id1 = albums::AlbumNoId { mbz_id, ..Faker.fake() };
        let album_no_id2 = albums::AlbumNoId { mbz_id, ..Faker.fake() };

        let album_id1 = upsert_album(temp_db.pool(), album_no_id1).await.unwrap();
        let album_id2 = upsert_album(temp_db.pool(), album_no_id2).await.unwrap();
        // Because they share the same mbz id
        assert_eq!(album_id1, album_id2);
    }

    #[tokio::test]
    async fn test_upsert_album_unique() {
        let temp_db = TemporaryDb::new_from_env().await;
        let album_no_id1 =
            albums::AlbumNoId { name: "album1".into(), mbz_id: None, ..Default::default() };
        let album_no_id2 =
            albums::AlbumNoId { name: "album1".into(), mbz_id: None, ..Default::default() };
        println!("{:?} {:?}", &album_no_id1, &album_no_id2);

        let album_id1 = upsert_album(temp_db.pool(), album_no_id1).await.unwrap();
        let album_id2 = upsert_album(temp_db.pool(), album_no_id2).await.unwrap();
        // Because they share the same property and mbz id is null.
        assert_eq!(album_id1, album_id2);

        let album_no_id1 = albums::AlbumNoId {
            name: "album2".into(),
            mbz_id: Some(Faker.fake()),
            ..Default::default()
        };
        let album_no_id2 = albums::AlbumNoId {
            name: "album2".into(),
            mbz_id: Some(Faker.fake()),
            ..Default::default()
        };
        let album_id1 = upsert_album(temp_db.pool(), album_no_id1).await.unwrap();
        let album_id2 = upsert_album(temp_db.pool(), album_no_id2).await.unwrap();
        // Because they share the same property but their mbz ids are different.
        assert_ne!(album_id1, album_id2);
    }
}
