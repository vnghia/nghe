use crate::{models::*, DatabasePool};

use anyhow::Result;
use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
use itertools::Itertools;
use uuid::Uuid;

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

pub async fn upsert_song<'a>(
    pool: &DatabasePool,
    song_id: Option<Uuid>,
    new_or_update_song: songs::NewOrUpdateSong<'a>,
) -> Result<Uuid> {
    if (song_id.is_some() && new_or_update_song.relative_path.is_some())
        || (song_id.is_none() && new_or_update_song.relative_path.is_none())
    {
        unreachable!("id (updating) or path (inserting) is mutually exclusive")
    }
    let song_id = if let Some(song_id) = song_id {
        diesel::update(songs::table)
            .filter(songs::id.eq(song_id))
            .set(new_or_update_song)
            .returning(songs::id)
            .get_result::<Uuid>(&mut pool.get().await?)
            .await?
    } else {
        diesel::insert_into(songs::table)
            .values(new_or_update_song)
            .returning(songs::id)
            .get_result::<Uuid>(&mut pool.get().await?)
            .await?
    };
    Ok(song_id)
}

#[cfg(test)]
mod tests {
    use super::super::album::upsert_album;
    use super::*;
    use crate::utils::test::media::query_all_song_information;
    use crate::utils::{song::test::SongTag, test::setup::setup_users_and_music_folders};

    use fake::{Fake, Faker};

    #[tokio::test]
    async fn test_upsert_song_insert() {
        let (temp_db, _, _temp_fs, music_folders) =
            setup_users_and_music_folders(1, 1, &[true]).await;

        let song_tag = Faker.fake::<SongTag>();
        let album_id = upsert_album(temp_db.pool(), (&song_tag.album).into())
            .await
            .unwrap();

        let song_path = Faker.fake::<String>();
        let song_hash: u64 = rand::random();
        let song_size: u64 = rand::random();

        let song_id = upsert_song(
            temp_db.pool(),
            None,
            song_tag.to_information().to_new_or_update_song(
                music_folders[0].id,
                album_id,
                song_hash,
                song_size,
                Some(&song_path),
            ),
        )
        .await
        .unwrap();

        let song_db_info = query_all_song_information(temp_db.pool(), song_id).await;

        assert_eq!(song_tag.title, song_db_info.tag.title);
        assert_eq!(song_path, song_db_info.relative_path);
        assert_eq!(song_hash, song_db_info.file_hash);
        assert_eq!(song_size, song_db_info.file_size);
    }

    #[tokio::test]
    async fn test_upsert_song_update() {
        let (temp_db, _, _temp_fs, music_folders) =
            setup_users_and_music_folders(1, 1, &[true]).await;

        let song_tag = Faker.fake::<SongTag>();
        let album_id = upsert_album(temp_db.pool(), (&song_tag.album).into())
            .await
            .unwrap();

        let song_path = Faker.fake::<String>();
        let song_hash: u64 = rand::random();
        let song_size: u64 = rand::random();

        let song_id = upsert_song(
            temp_db.pool(),
            None,
            song_tag.to_information().to_new_or_update_song(
                music_folders[0].id,
                album_id,
                song_hash,
                song_size,
                Some(&song_path),
            ),
        )
        .await
        .unwrap();

        let new_song_tag = Faker.fake::<SongTag>();
        let new_album_id = upsert_album(temp_db.pool(), (&new_song_tag.album).into())
            .await
            .unwrap();

        let new_song_hash: u64 = rand::random();
        let new_song_size: u64 = rand::random();

        let new_song_id = upsert_song(
            temp_db.pool(),
            Some(song_id),
            new_song_tag.to_information().to_new_or_update_song(
                music_folders[0].id,
                new_album_id,
                new_song_hash,
                new_song_size,
                Option::<&String>::None,
            ),
        )
        .await
        .unwrap();

        assert_eq!(song_id, new_song_id);
        let song_db_info = query_all_song_information(temp_db.pool(), new_song_id).await;

        assert_eq!(new_song_tag.title, song_db_info.tag.title);
        assert_eq!(song_path, song_db_info.relative_path);
        assert_eq!(new_song_hash, song_db_info.file_hash);
        assert_eq!(new_song_size, song_db_info.file_size);
    }
}
