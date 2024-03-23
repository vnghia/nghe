use anyhow::Result;
use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
use itertools::Itertools;
use uuid::Uuid;

use crate::models::*;
use crate::DatabasePool;

pub async fn upsert_song_artists(
    pool: &DatabasePool,
    song_id: &Uuid,
    artist_ids: &[Uuid],
) -> Result<()> {
    diesel::insert_into(songs_artists::table)
        .values(
            artist_ids
                .iter()
                .map(|artist_id| songs_artists::NewSongArtist { song_id, artist_id })
                .collect_vec(),
        )
        .on_conflict_do_nothing()
        .execute(&mut pool.get().await?)
        .await?;
    Ok(())
}

pub async fn insert_song<'a>(
    pool: &DatabasePool,
    information_db: songs::SongFullInformationDB<'a>,
) -> Result<Uuid> {
    diesel::insert_into(songs::table)
        .values(information_db)
        .returning(songs::id)
        .get_result::<Uuid>(&mut pool.get().await?)
        .await
        .map_err(anyhow::Error::from)
}

pub async fn update_song<'a>(
    pool: &DatabasePool,
    id: Uuid,
    information_db: songs::SongUpdateInformationDB<'a>,
) -> Result<()> {
    diesel::update(songs::table)
        .filter(songs::id.eq(id))
        .set(information_db)
        .execute(&mut pool.get().await?)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};

    use super::super::album::upsert_album;
    use super::*;
    use crate::utils::song::test::SongTag;
    use crate::utils::test::media::query_all_song_information;
    use crate::utils::test::setup::TestInfra;

    #[tokio::test]
    async fn test_insert_song() {
        let test_infra = TestInfra::setup_users_and_music_folders(1, 1, &[true]).await;

        let song_tag = Faker.fake::<SongTag>();
        let album_id = upsert_album(test_infra.pool(), (&song_tag.album).into()).await.unwrap();

        let song_path = Faker.fake::<String>();
        let song_hash: i64 = rand::random();
        let song_size: i64 = rand::random();

        let song_id = insert_song(
            test_infra.pool(),
            song_tag.to_information().to_full_information_db(
                album_id,
                song_hash,
                song_size,
                test_infra.music_folder_ids(0..=0)[0],
                &song_path,
            ),
        )
        .await
        .unwrap();

        let song_db_info = query_all_song_information(test_infra.pool(), song_id).await;

        assert_eq!(song_tag.title, song_db_info.tag.title);
        assert_eq!(song_path, song_db_info.relative_path);
        assert_eq!(song_hash as u64, song_db_info.file_hash);
        assert_eq!(song_size as u64, song_db_info.file_size);
    }

    #[tokio::test]
    async fn test_update_song() {
        let test_infra = TestInfra::setup_users_and_music_folders(1, 1, &[true]).await;

        let song_tag = Faker.fake::<SongTag>();
        let album_id = upsert_album(test_infra.pool(), (&song_tag.album).into()).await.unwrap();

        let song_path = Faker.fake::<String>();
        let song_hash: i64 = rand::random();
        let song_size: i64 = rand::random();

        let song_id = insert_song(
            test_infra.pool(),
            song_tag.to_information().to_full_information_db(
                album_id,
                song_hash,
                song_size,
                test_infra.music_folder_ids(0..=0)[0],
                &song_path,
            ),
        )
        .await
        .unwrap();

        let new_song_tag = Faker.fake::<SongTag>();
        let new_album_id =
            upsert_album(test_infra.pool(), (&new_song_tag.album).into()).await.unwrap();

        let new_song_hash: i64 = rand::random();
        let new_song_size: i64 = rand::random();

        update_song(
            test_infra.pool(),
            song_id,
            new_song_tag.to_information().to_update_information_db(
                new_album_id,
                new_song_hash,
                new_song_size,
            ),
        )
        .await
        .unwrap();

        let song_db_info = query_all_song_information(test_infra.pool(), song_id).await;

        assert_eq!(new_song_tag.title, song_db_info.tag.title);
        assert_eq!(song_path, song_db_info.relative_path);
        assert_eq!(new_song_hash as u64, song_db_info.file_hash);
        assert_eq!(new_song_size as u64, song_db_info.file_size);
    }
}
