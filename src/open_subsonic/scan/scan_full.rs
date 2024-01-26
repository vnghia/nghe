use super::{
    album::upsert_album, artist::upsert_artists, song::refresh_song_artists, song::upsert_song,
};
use crate::{
    models::*,
    utils::{fs::files::scan_media_files, song::tag::SongTag},
    DatabasePool, OSResult,
};

use diesel::{ExpressionMethods, OptionalExtension};
use diesel_async::RunQueryDsl;
use uuid::Uuid;
use xxhash_rust::xxh3::xxh3_64;

pub async fn scan_full<T: AsRef<str>>(
    pool: &DatabasePool,
    ignored_prefixes: &[T],
    music_folders: &[music_folders::MusicFolder],
) -> OSResult<()> {
    for music_folder in music_folders {
        let music_folder_path = music_folder.path.clone();
        for (song_absolute_path, song_relative_path, song_file_type, song_file_size) in
            tokio::task::spawn_blocking(move || scan_media_files(music_folder_path)).await??
        {
            let song_file_metadata_db = diesel::update(songs::table)
                .filter(songs::music_folder_id.eq(music_folder.id))
                .filter(songs::path.eq(&song_relative_path))
                .set(songs::scanned_at.eq(time::OffsetDateTime::now_utc()))
                .returning((songs::id, songs::file_hash, songs::file_size))
                .get_result::<(Uuid, i64, i64)>(&mut pool.get().await?)
                .await
                .optional()?;

            let song_data = tokio::fs::read(&song_absolute_path).await?;
            let (song_file_hash, song_data) =
                tokio::task::spawn_blocking(move || (xxh3_64(&song_data), song_data)).await?;

            let song_id = if let Some((song_id_db, song_file_hash_db, song_file_size_db)) =
                song_file_metadata_db
            {
                // there is already an entry in the database with the same music folder and relative path
                // and it has the same size and hash with the file on local disk, continue.
                if song_file_size_db as u64 == song_file_size
                    && song_file_hash_db as u64 == song_file_hash
                {
                    continue;
                }
                Some(song_id_db)
            } else {
                None
            };

            let song_tag = SongTag::parse(&song_data, song_file_type)?;

            let artist_ids = upsert_artists(pool, ignored_prefixes, &song_tag.artists).await?;
            let album_id = upsert_album(pool, std::borrow::Cow::Borrowed(&song_tag.album)).await?;

            let song_id = upsert_song(
                pool,
                song_id,
                song_tag.into_new_or_update_song(
                    music_folder.id,
                    album_id,
                    song_file_hash,
                    song_file_size,
                    // only supply path if song id is none
                    // i.e: we are inserting a new song.
                    if song_id.is_none() {
                        Some(&song_relative_path)
                    } else {
                        None
                    },
                ),
            )
            .await?;
            refresh_song_artists(pool, song_id, &artist_ids).await?;
        }
    }

    tracing::info!("done scanning songs");
    Ok(())
}
