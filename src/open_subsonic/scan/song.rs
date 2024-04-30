use std::path::Path;

use anyhow::Result;
use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
use itertools::Itertools;
use lofty::picture::Picture;
use uuid::Uuid;

use crate::models::*;
use crate::open_subsonic::cover_art::upsert_cover_art_from_data;
use crate::utils::song::file_type::picture_to_extension;
use crate::{DatabasePool, OSError};

pub async fn upsert_song_artists(
    pool: &DatabasePool,
    song_id: Uuid,
    artist_ids: &[Uuid],
) -> Result<()> {
    diesel::insert_into(songs_artists::table)
        .values(
            artist_ids
                .iter()
                .copied()
                .map(|artist_id| songs_artists::NewSongArtist { song_id, artist_id })
                .collect_vec(),
        )
        .on_conflict((songs_artists::song_id, songs_artists::artist_id))
        .do_update()
        .set(songs_artists::upserted_at.eq(time::OffsetDateTime::now_utc()))
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

pub async fn upsert_song_cover_art<P: AsRef<Path>>(
    pool: &DatabasePool,
    picture: &Picture,
    song_art_dir: P,
) -> Result<Uuid> {
    upsert_cover_art_from_data(
        pool,
        song_art_dir,
        picture.data(),
        picture_to_extension(
            picture
                .mime_type()
                .ok_or_else(|| OSError::InvalidParameter("Picture format".into()))?,
        ),
    )
    .await
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};

    use super::super::album::upsert_album;
    use super::*;
    use crate::utils::song::test::SongTag;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_insert_song() {
        let infra = Infra::new().await.n_folder(1).await.add_user(None).await;

        let song_tag = Faker.fake::<SongTag>();
        let album_id = upsert_album(infra.pool(), (&song_tag.album).into()).await.unwrap();

        let song_path = Infra::fake_fs_name();
        let song_hash: i64 = rand::random();
        let song_size: i32 = rand::random();

        let song_id = insert_song(
            infra.pool(),
            song_tag.to_information().to_full_information_db(
                album_id,
                song_hash,
                song_size,
                None,
                infra.music_folder_id(0),
                &song_path,
            ),
        )
        .await
        .unwrap();

        let song_db_info = infra.song_db_info(song_id).await;

        assert_eq!(song_tag.song.name, song_db_info.tag.song.name);
        assert_eq!(song_path, song_db_info.relative_path);
        assert_eq!(song_hash as u64, song_db_info.file_hash);
        assert_eq!(song_size as u32, song_db_info.file_size);
    }

    #[tokio::test]
    async fn test_update_song() {
        let infra = Infra::new().await.n_folder(1).await.add_user(None).await;

        let song_tag = Faker.fake::<SongTag>();
        let album_id = upsert_album(infra.pool(), (&song_tag.album).into()).await.unwrap();

        let song_path = Infra::fake_fs_name();
        let song_hash: i64 = rand::random();
        let song_size: i32 = rand::random();

        let song_id = insert_song(
            infra.pool(),
            song_tag.to_information().to_full_information_db(
                album_id,
                song_hash,
                song_size,
                None,
                infra.music_folder_id(0),
                &song_path,
            ),
        )
        .await
        .unwrap();

        let new_song_tag = Faker.fake::<SongTag>();
        let new_album_id = upsert_album(infra.pool(), (&new_song_tag.album).into()).await.unwrap();

        let new_song_hash: i64 = rand::random();
        let new_song_size: i32 = rand::random();

        update_song(
            infra.pool(),
            song_id,
            new_song_tag.to_information().to_update_information_db(
                new_album_id,
                new_song_hash,
                new_song_size,
                None,
            ),
        )
        .await
        .unwrap();

        let song_db_info = infra.song_db_info(song_id).await;

        assert_eq!(new_song_tag.song.name, song_db_info.tag.song.name);
        assert_eq!(song_path, song_db_info.relative_path);
        assert_eq!(new_song_hash as u64, song_db_info.file_hash);
        assert_eq!(new_song_size as u32, song_db_info.file_size);
    }
}
