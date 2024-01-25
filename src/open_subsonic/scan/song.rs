use std::path::Path;

use super::{album::upsert_album, artist::upsert_artists};
use crate::models::*;
use crate::utils::song::tag::SongTag;
use crate::{DatabasePool, OSResult};

use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
use itertools::Itertools;
use uuid::Uuid;

pub async fn refresh_song_artists(
    pool: &DatabasePool,
    song_id: Uuid,
    artist_ids: &[Uuid],
) -> OSResult<()> {
    let upsert_start_time = time::OffsetDateTime::now_utc();
    diesel::insert_into(songs_artists::table)
        .values(
            artist_ids
                .iter()
                .cloned()
                .map(|artist_id| songs_artists::NewSongArtist {
                    song_id,
                    artist_id,
                    upserted_at: time::OffsetDateTime::now_utc(),
                })
                .collect_vec(),
        )
        .on_conflict_do_nothing()
        .execute(&mut pool.get().await?)
        .await?;
    diesel::delete(songs_artists::table)
        .filter(songs_artists::upserted_at.lt(upsert_start_time))
        .execute(&mut pool.get().await?)
        .await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn upsert_song<TI: AsRef<str>, TP: AsRef<Path>>(
    pool: &DatabasePool,
    ignored_prefixes: &[TI],
    music_folder_id: Uuid,
    song_id: Option<Uuid>,
    song_tag: SongTag,
    song_file_hash: u64,
    song_file_size: u64,
    song_relative_path: TP,
) -> OSResult<Uuid> {
    let artist_ids = upsert_artists(pool, ignored_prefixes, &song_tag.artists).await?;
    let album_id = upsert_album(pool, song_tag.album.into()).await?;

    let song_id = if let Some(song_id) = song_id {
        let update_song = songs::UpdateSong {
            id: song_id,
            title: song_tag.title.into(),
            album_id,
            music_folder_id,
            file_hash: song_file_hash as i64,
            file_size: song_file_size as i64,
        };
        diesel::update(&update_song)
            .set(&update_song)
            .returning(songs::id)
            .get_result::<Uuid>(&mut pool.get().await?)
            .await?
    } else {
        let new_song = songs::NewSong {
            title: song_tag.title.into(),
            album_id,
            music_folder_id,
            path: song_relative_path.as_ref().to_string_lossy(),
            file_hash: song_file_hash as i64,
            file_size: song_file_size as i64,
        };
        diesel::insert_into(songs::table)
            .values(&new_song)
            .returning(songs::id)
            .get_result::<Uuid>(&mut pool.get().await?)
            .await?
    };

    refresh_song_artists(pool, song_id, &artist_ids).await?;

    Ok(song_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        open_subsonic::browsing::test::setup_user_and_music_folders,
        utils::test::song::query_all_song_information,
    };

    use fake::{Fake, Faker};
    use itertools::Itertools;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_upsert_song_insert() {
        let (db, _, _, _temp_fs, music_folders, _) =
            setup_user_and_music_folders(1, 1, &[true]).await;

        let song_tag = Faker.fake::<SongTag>();
        let song_path = Faker.fake::<PathBuf>();
        let song_hash: u64 = rand::random();
        let song_size: u64 = rand::random();
        let song_id = upsert_song(
            db.get_pool(),
            &[""],
            music_folders[0].id,
            None,
            song_tag.clone(),
            song_hash,
            song_size,
            &song_path,
        )
        .await
        .unwrap();

        let (song, album, artists) = query_all_song_information(db.get_pool(), song_id).await;

        assert_eq!(song_tag.title, song.title);
        assert_eq!(song_hash, song.file_hash as u64);
        assert_eq!(song_size, song.file_size as u64);
        assert_eq!(song_tag.album, album.name);
        assert_eq!(
            song_tag.artists.into_iter().sorted().collect_vec(),
            artists
                .into_iter()
                .map(|artist| artist.name)
                .sorted()
                .collect_vec()
        );
    }

    #[tokio::test]
    async fn test_upsert_song_update() {
        let (db, _, _, _temp_fs, music_folders, _) =
            setup_user_and_music_folders(1, 1, &[true]).await;

        let song_tag = Faker.fake::<SongTag>();
        let song_path = Faker.fake::<PathBuf>();
        let song_hash: u64 = rand::random();
        let song_size: u64 = rand::random();
        let song_id = upsert_song(
            db.get_pool(),
            &[""],
            music_folders[0].id,
            None,
            song_tag.clone(),
            song_hash,
            song_size,
            &song_path,
        )
        .await
        .unwrap();

        let new_song_tag = Faker.fake::<SongTag>();
        let new_song_hash: u64 = rand::random();
        let new_song_size: u64 = rand::random();
        let new_song_id = upsert_song(
            db.get_pool(),
            &[""],
            music_folders[0].id,
            Some(song_id),
            new_song_tag.clone(),
            new_song_hash,
            new_song_size,
            &song_path,
        )
        .await
        .unwrap();

        assert_eq!(song_id, new_song_id);
        let (song, album, artists) = query_all_song_information(db.get_pool(), new_song_id).await;

        assert_eq!(new_song_tag.title, song.title);
        assert_eq!(new_song_tag.album, album.name);
        assert_eq!(new_song_hash, song.file_hash as u64);
        assert_eq!(new_song_size, song.file_size as u64);
        assert_eq!(
            new_song_tag.artists.into_iter().sorted().collect_vec(),
            artists
                .into_iter()
                .map(|artist| artist.name)
                .sorted()
                .collect_vec()
        );
    }
}
